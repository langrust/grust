//! HIR [`stream::Expr`][Expr] module.

prelude! {
    graph::{Label, DiGraphMap},
}

pub type Stmt = hir::Stmt<Expr>;

impl Stmt {
    pub fn get_identifiers(&self) -> Vec<usize> {
        let mut identifiers = match &self.expression.kind {
            stream::Kind::Expression { expression } => match expression {
                expr::Kind::Match { arms, .. } => arms
                    .iter()
                    .flat_map(|(pattern, _, statements, _)| {
                        statements
                            .iter()
                            .flat_map(|statement| statement.get_identifiers())
                            .chain(pattern.identifiers())
                    })
                    .collect(),
                _ => vec![],
            },
            _ => vec![],
        };

        identifiers.append(&mut self.pattern.identifiers());
        identifiers
    }

    /// Increments memory with statement's expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications. Transform followed
    /// by expressions in signal call.
    ///
    /// # Example
    ///
    /// A statement `x: int = 0 fby v;` increments memory with the buffer `mem: int = 0 fby v;` and
    /// becomes `x: int = mem;`.
    ///
    /// A statement `x: int = my_node(s, x_1).o;` increments memory with the node call
    /// `mem_my_node_o_: (my_node, o);` and the statement is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        self.expression
            .memorize(identifier_creator, memory, contract, symbol_table)
    }

    /// Change HIR statement into a normal form.
    ///
    /// The normal form of an statement is as follows:
    ///
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normal_form(
        self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<stream::Stmt> {
        let Stmt {
            pattern,
            mut expression,
            location,
        } = self;

        // change expression into normal form and get additional statements
        let mut statements = match expression.kind {
            stream::Kind::NodeApplication {
                called_node_id,
                ref mut inputs,
                ..
            } => {
                let new_statements = inputs
                    .iter_mut()
                    .flat_map(|(_, expression)| {
                        expression.into_signal_call(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect::<Vec<_>>();

                // change dependencies to be the sum of inputs dependencies
                let reduced_graph = nodes_reduced_graphs.get(&called_node_id).unwrap();
                expression.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(input_id, expression)| {
                            symbol_table
                                .get_node_outputs(called_node_id)
                                .iter()
                                .flat_map(|(_, output_id)| {
                                    reduced_graph.edge_weight(*output_id, *input_id).map_or(
                                        vec![],
                                        |label1| {
                                            expression
                                                .get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .map(|(id, label2)| (id, label1.add(&label2)))
                                                .collect()
                                        },
                                    )
                                })
                        })
                        .collect(),
                );

                new_statements
            }
            _ => expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table),
        };

        // recreate the new statement with modified expression
        // todo: isn't it equal to self?
        let normal_formed_statement = Stmt {
            pattern,
            expression,
            location,
        };

        // push normal_formed statement in the statements storage (in scheduling order)
        statements.push(normal_formed_statement);

        // return statements
        statements
    }

    pub fn add_to_graph(&self, graph: &mut DiGraphMap<usize, Label>) {
        let Stmt {
            pattern,
            expression,
            ..
        } = self;
        let signals = pattern.identifiers();
        for from in signals.iter() {
            for (to, label) in expression.get_dependencies() {
                graph::add_edge(graph, *from, *to, label.clone());
            }
        }
        match &self.expression.kind {
            stream::Kind::Expression { expression } => match expression {
                hir::expr::Kind::Match { arms, .. } => {
                    arms.iter().for_each(|(_, bound, statements, _)| {
                        if let Some(bound) = bound {
                            for from in signals.iter() {
                                for (to, label) in bound.get_dependencies() {
                                    graph::add_edge(graph, *from, *to, label.clone());
                                }
                            }
                        }
                        statements
                            .iter()
                            .for_each(|statement| statement.add_to_graph(graph))
                    })
                }
                _ => (),
            },
            _ => (),
        }
    }

    /// Add the statement identifier to the identifier creator.
    ///
    /// It will add the statement identifier to the identifier creator. If the identifier already
    /// exists, then the new identifier created by the identifier creator will be added to the
    /// renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<usize, Either<usize, stream::Expr>>,
        symbol_table: &mut SymbolTable,
    ) {
        // create fresh identifiers for the new statement
        let local_signals = self.pattern.identifiers();
        for signal_id in local_signals {
            let name = symbol_table.get_name(signal_id);
            let scope = symbol_table.get_scope(signal_id).clone();
            let fresh_name = identifier_creator.new_identifier(name);
            if Scope::Output != scope && &fresh_name != name {
                let typing = Some(symbol_table.get_type(signal_id).clone());
                let fresh_id = symbol_table.insert_fresh_signal(fresh_name, scope, typing);
                let _unique = context_map.insert(signal_id, Either::Left(fresh_id));
                debug_assert!(_unique.is_none());
            }
        }
    }

    /// Replace identifier occurrence by element in context.
    ///
    /// It will return a new statement where the expression has been modified according to the
    /// context:
    ///
    /// - if an identifier is mapped to another identifier, then rename all occurrence of the
    ///   identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to the identifier by
    ///   the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function with the statement `z =
    /// x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<usize, Either<usize, stream::Expr>>,
    ) -> stream::Stmt {
        let mut new_statement = self.clone();

        // replace statement's identifiers by the new ones
        let local_signals = new_statement.pattern.identifiers_mut();
        for signal_id in local_signals {
            if let Some(element) = context_map.get(&signal_id) {
                match element {
                    Either::Left(new_id)
                    | Either::Right(stream::Expr {
                        kind:
                            stream::Kind::Expression {
                                expression: hir::expr::Kind::Identifier { id: new_id },
                            },
                        ..
                    }) => {
                        *signal_id = new_id.clone();
                    }
                    Either::Right(_) => unreachable!(),
                }
            }
        }

        // replace identifiers in statement's expression
        new_statement.expression.replace_by_context(context_map);

        new_statement
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// ```
    /// In this example, an statement `fib: int = semi_fib(fib).o` calls
    /// `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed_recursive(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
        nodes: &HashMap<usize, Component>,
    ) -> Vec<stream::Stmt> {
        let mut current_statements = vec![self.clone()];
        let mut new_statements =
            self.inline_when_needed(memory, identifier_creator, symbol_table, nodes);
        while current_statements != new_statements {
            current_statements = new_statements;
            new_statements = current_statements
                .clone()
                .into_iter()
                .flat_map(|statement| {
                    statement.inline_when_needed(memory, identifier_creator, symbol_table, nodes)
                })
                .collect();
        }
        new_statements
    }

    fn inline_when_needed(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
        nodes: &HashMap<usize, Component>,
    ) -> Vec<stream::Stmt> {
        match &self.expression.kind {
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                memory_id,
                ..
            } => {
                // a loop in the graph induces that "node call" inputs depends on output
                let is_loop = {
                    let mut graph = DiGraphMap::new();
                    let outs = self.pattern.identifiers();
                    let in_deps = inputs.iter().flat_map(|(_, expr)| expr.get_dependencies());
                    for (to, label) in in_deps {
                        for from in outs.iter() {
                            graph.add_edge(*from, *to, label.clone());
                        }
                    }
                    graph::toposort(&graph, None).is_err()
                };

                // then node call must be inlined
                if is_loop {
                    let called_unitary_node = nodes.get(&called_node_id).unwrap();

                    // get statements from called node, with corresponding inputs
                    let (retrieved_statements, retrieved_memory) = called_unitary_node
                        .instantiate_statements_and_memory(
                            identifier_creator,
                            inputs,
                            Some(self.pattern),
                            symbol_table,
                        );

                    // remove called node from memory
                    memory.remove_called_node(memory_id.unwrap());

                    memory.combine(retrieved_memory);
                    retrieved_statements
                } else {
                    // otherwise, just return self
                    vec![self]
                }
            }
            _ => vec![self],
        }
    }
}

pub type ExprKind = expr::Kind<Expr>;

impl ExprKind {
    /// Increment memory with expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications. Transform followed
    /// by expressions in signal call.
    ///
    /// # Example
    ///
    /// An expression `0 fby v` increments memory with the buffer `mem: int = 0 fby v;` and becomes
    /// a call to `mem`.
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the node call `mem_my_node_o_:
    /// (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        match self {
            Self::Constant { .. }
            | Self::Identifier { .. }
            | Self::Abstraction { .. }
            | Self::Enumeration { .. } => (),
            Self::UnOp { expression, .. } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                left_expression.memorize(identifier_creator, memory, contract, symbol_table);
                right_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                true_expression.memorize(identifier_creator, memory, contract, symbol_table);
                false_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Application {
                function_expression,
                inputs,
            } => {
                function_expression.memorize(identifier_creator, memory, contract, symbol_table);
                inputs.iter_mut().for_each(|expression| {
                    expression.memorize(identifier_creator, memory, contract, symbol_table)
                })
            }
            Self::Structure { fields, .. } => fields.iter_mut().for_each(|(_, expression)| {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }),
            Self::Array { elements } | Self::Tuple { elements } => {
                elements.iter_mut().for_each(|expression| {
                    expression.memorize(identifier_creator, memory, contract, symbol_table)
                })
            }
            Self::Match { expression, arms } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                arms.iter_mut().for_each(|(_, option, block, expression)| {
                    option.as_mut().map(|expression| {
                        expression.memorize(identifier_creator, memory, contract, symbol_table)
                    });
                    block.iter_mut().for_each(|statement| {
                        statement.memorize(identifier_creator, memory, contract, symbol_table)
                    });
                    expression.memorize(identifier_creator, memory, contract, symbol_table)
                })
            }
            Self::FieldAccess { expression, .. } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::TupleElementAccess { expression, .. } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Map {
                expression,
                function_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                function_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                initialization_expression.memorize(
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                function_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Sort {
                expression,
                function_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                function_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Zip { arrays } => arrays.iter_mut().for_each(|expression| {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }),
        }
    }

    /// Change HIR expression into a normal form.
    ///
    /// The normal form of an expression is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normal_form(
        &mut self,
        dependencies: &mut Dependencies,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<stream::Stmt> {
        match self {
            Self::Constant { .. }
            | Self::Identifier { .. }
            | Self::Enumeration { .. }
            | Self::Abstraction { .. } => {
                vec![]
            }
            Self::UnOp { expression, .. } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                *dependencies = Dependencies::from(expression.get_dependencies().clone());

                new_statements
            }

            Self::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                let mut new_statements = left_expression.normal_form(
                    nodes_reduced_graphs,
                    identifier_creator,
                    symbol_table,
                );
                let mut other_statements = right_expression.normal_form(
                    nodes_reduced_graphs,
                    identifier_creator,
                    symbol_table,
                );
                new_statements.append(&mut other_statements);

                let mut expression_dependencies = left_expression.get_dependencies().clone();
                let mut other_dependencies = right_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);

                new_statements
            }

            Self::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                let mut new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);
                let mut other_statements = true_expression.normal_form(
                    nodes_reduced_graphs,
                    identifier_creator,
                    symbol_table,
                );
                new_statements.append(&mut other_statements);
                let mut other_statements = false_expression.normal_form(
                    nodes_reduced_graphs,
                    identifier_creator,
                    symbol_table,
                );
                new_statements.append(&mut other_statements);

                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut other_dependencies = true_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);
                let mut other_dependencies = false_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);

                new_statements
            }

            Self::Application { ref mut inputs, .. } => {
                let new_statements = inputs
                    .iter_mut()
                    .flat_map(|expression| {
                        expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect();

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );

                new_statements
            }

            Self::Structure { fields, .. } => {
                let new_statements = fields
                    .iter_mut()
                    .flat_map(|(_, expression)| {
                        expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect();

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );

                new_statements
            }
            Self::Array { elements } | Self::Tuple { elements } => {
                let new_statements = elements
                    .iter_mut()
                    .flat_map(|expression| {
                        expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect();

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );

                new_statements
            }
            Self::Match {
                expression, arms, ..
            } => {
                let mut statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(pattern, bound, body, matched_expression)| {
                        // get local signals defined in pattern
                        let local_signals = pattern.identifiers();

                        // normalize body statements
                        *body = body
                            .iter()
                            .flat_map(|statement| {
                                statement.clone().normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    symbol_table,
                                )
                            })
                            .collect();

                        // remove identifiers created by the pattern from the dependencies
                        let (mut bound_statements, mut bound_dependencies) =
                            bound.as_mut().map_or((vec![], vec![]), |expression| {
                                let stmts = expression.normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    symbol_table,
                                );
                                (
                                    stmts,
                                    expression
                                        .get_dependencies()
                                        .clone()
                                        .into_iter()
                                        .filter(|(signal, _)| !local_signals.contains(signal))
                                        .collect(),
                                )
                            });
                        statements.append(&mut bound_statements);
                        expression_dependencies.append(&mut bound_dependencies);

                        let mut matched_expression_statements = matched_expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        let mut matched_expression_dependencies = matched_expression
                            .get_dependencies()
                            .clone()
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect();
                        body.append(&mut matched_expression_statements);
                        expression_dependencies.append(&mut matched_expression_dependencies)
                    });

                *dependencies = Dependencies::from(expression_dependencies);

                statements
            }
            Self::FieldAccess { expression, .. } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                *dependencies = Dependencies::from(expression.get_dependencies().clone());

                new_statements
            }
            Self::TupleElementAccess { expression, .. } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                *dependencies = Dependencies::from(expression.get_dependencies().clone());

                new_statements
            }
            Self::Map { expression, .. } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                *dependencies = Dependencies::from(expression.get_dependencies().clone());

                new_statements
            }
            Self::Fold {
                expression,
                initialization_expression,
                ..
            } => {
                let mut new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);
                let mut initialization_statements = initialization_expression.normal_form(
                    nodes_reduced_graphs,
                    identifier_creator,
                    symbol_table,
                );
                new_statements.append(&mut initialization_statements);

                // get matched expressions dependencies
                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut initialization_expression_dependencies =
                    expression.get_dependencies().clone();
                expression_dependencies.append(&mut initialization_expression_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);

                new_statements
            }
            Self::Sort { expression, .. } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                *dependencies = Dependencies::from(expression.get_dependencies().clone());

                new_statements
            }
            Self::Zip { arrays, .. } => {
                let new_statements = arrays
                    .iter_mut()
                    .flat_map(|array| {
                        array.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table)
                    })
                    .collect();

                *dependencies = Dependencies::from(
                    arrays
                        .iter()
                        .flat_map(|array| array.get_dependencies().clone())
                        .collect(),
                );

                new_statements
            }
        }
    }

    /// Replace identifier occurrence by element in context.
    ///
    /// It will modify the expression according to the context:
    ///
    /// - if an identifier is mapped to another identifier, then rename all occurrence of the
    ///   identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to the identifier by
    ///   the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` will become `a + b/2`.
    pub fn replace_by_context(
        &mut self,
        dependencies: &mut Dependencies,
        context_map: &HashMap<usize, Either<usize, stream::Expr>>,
    ) -> Option<stream::Expr> {
        match self {
            Self::Constant { .. } | Self::Abstraction { .. } | Self::Enumeration { .. } => None,
            Self::Identifier { ref mut id } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Either::Left(new_id) => {
                            *id = *new_id;
                            *dependencies = Dependencies::from(vec![(*new_id, Label::Weight(0))]);
                            None
                        }
                        Either::Right(new_expression) => Some(new_expression.clone()),
                    }
                } else {
                    None
                }
            }
            Self::UnOp { expression, .. } => {
                expression.replace_by_context(context_map);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
                None
            }
            Self::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                left_expression.replace_by_context(context_map);
                right_expression.replace_by_context(context_map);

                let mut expression_dependencies = left_expression.get_dependencies().clone();
                let mut other_dependencies = right_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                expression.replace_by_context(context_map);
                true_expression.replace_by_context(context_map);
                false_expression.replace_by_context(context_map);

                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut other_dependencies = true_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);
                let mut other_dependencies = false_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Application { ref mut inputs, .. } => {
                inputs
                    .iter_mut()
                    .for_each(|expression| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            Self::Structure { ref mut fields, .. } => {
                fields
                    .iter_mut()
                    .for_each(|(_, expression)| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            Self::Array { ref mut elements } | Self::Tuple { ref mut elements } => {
                elements
                    .iter_mut()
                    .for_each(|expression| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            Self::Match {
                ref mut expression,
                ref mut arms,
                ..
            } => {
                expression.replace_by_context(context_map);
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(pattern, bound, body, matched_expression)| {
                        let local_signals = pattern.identifiers();

                        // remove identifiers created by the pattern from the context
                        let context_map = context_map
                            .clone()
                            .into_iter()
                            .filter(|(key, _)| !local_signals.contains(key))
                            .collect();

                        if let Some(expression) = bound.as_mut() {
                            expression.replace_by_context(&context_map);
                            let mut bound_dependencies = expression
                                .get_dependencies()
                                .clone()
                                .into_iter()
                                .filter(|(signal, _)| !local_signals.contains(signal))
                                .collect();
                            expression_dependencies.append(&mut bound_dependencies);
                        };

                        body.iter_mut().for_each(|statement| {
                            statement.expression.replace_by_context(&context_map)
                        });

                        matched_expression.replace_by_context(&context_map);
                        let mut matched_expression_dependencies = matched_expression
                            .get_dependencies()
                            .clone()
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect::<Vec<(usize, Label)>>();
                        expression_dependencies.append(&mut matched_expression_dependencies);
                    });

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::FieldAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::TupleElementAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Map {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Fold {
                ref mut expression,
                ref mut initialization_expression,
                ..
            } => {
                expression.replace_by_context(context_map);
                initialization_expression.replace_by_context(context_map);
                // get matched expressions dependencies
                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut initialization_expression_dependencies =
                    expression.get_dependencies().clone();
                expression_dependencies.append(&mut initialization_expression_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Sort {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Zip { ref mut arrays, .. } => {
                arrays
                    .iter_mut()
                    .for_each(|array| array.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    arrays
                        .iter()
                        .flat_map(|array| array.get_dependencies().clone())
                        .collect(),
                );
                None
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression kind AST.
pub enum Kind {
    /// Expression.
    Expression {
        /// The expression kind.
        expression: expr::Kind<Expr>,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The buffered id.
        id: usize,
        /// The initialization constant.
        constant: Box<Expr>,
    },
    /// Node application stream expression.
    NodeApplication {
        /// Node's id in memory.
        memory_id: Option<usize>,
        /// Called node's id in Symbol Table.
        called_node_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, Expr)>,
    },
    /// Detect a rising edge of the expression.
    RisingEdge {
        /// The expression to detect the rising edge from.
        expression: Box<Expr>,
    },
    /// Present event expression.
    SomeEvent {
        /// The expression of the event.
        expression: Box<Expr>,
    },
    /// Absent event expression.
    NoneEvent,
}

mk_new! { impl Kind =>
    Expression: expr { expression: expr::Kind<Expr> }
    FollowedBy: fby {
        id: usize,
        constant: Expr = constant.into(),
    }
    NodeApplication: call {
        memory_id = None,
        called_node_id: usize,
        inputs: Vec<(usize, Expr)>,
    }
    RisingEdge: rising_edge {
        expression: Expr = expression.into(),
    }
    SomeEvent: some_event {
        expression: Expr = expression.into(),
    }
    NoneEvent: none_event ()
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression AST.
pub struct Expr {
    /// Stream expression kind.
    pub kind: Kind,
    /// Stream expression type.
    pub typing: Option<Typ>,
    /// Stream expression location.
    pub location: Location,
    /// Stream expression dependencies.
    pub dependencies: Dependencies,
}

/// Constructs stream expression.
///
/// Typing, location and dependencies are empty.
pub fn expr(kind: Kind) -> Expr {
    Expr {
        kind,
        typing: None,
        location: Location::default(),
        dependencies: Dependencies::new(),
    }
}

impl Expr {
    /// Get stream expression's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    /// Get stream expression's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
    /// Get stream expression's dependencies.
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }

    /// Tell if it is in normal form.
    ///
    /// - component application as root expression
    /// - no rising edge
    pub fn is_normal_form(&self) -> bool {
        let predicate_expression = |expression: &Expr| {
            expression.no_component_application() && expression.no_rising_edge()
        };
        let predicate_statement = |statement: &Stmt| statement.expression.is_normal_form();
        match &self.kind {
            Kind::Expression { expression } => {
                expression.propagate_predicate(predicate_expression, predicate_statement)
            }
            Kind::FollowedBy { .. } => true,
            Kind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| predicate_expression(expression)),
            Kind::SomeEvent { expression } => predicate_expression(expression),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => false,
        }
    }

    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_component_application, |statement| {
                    statement.expression.no_component_application()
                }),
            Kind::FollowedBy { .. } => true,
            Kind::NodeApplication { .. } => false,
            Kind::SomeEvent { expression } => expression.no_component_application(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { expression } => expression.no_component_application(),
        }
    }

    /// Tell if there is no rising edge.
    pub fn no_rising_edge(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_rising_edge, |statement| {
                    statement.expression.no_rising_edge()
                }),
            Kind::FollowedBy { .. } => true,
            Kind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_rising_edge()),
            Kind::SomeEvent { expression } => expression.no_rising_edge(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => false,
        }
    }

    /// Increment memory with expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications. Transform followed
    /// by expressions in signal call.
    ///
    /// # Example
    ///
    /// An expression `0 fby v` increments memory with the buffer `mem: int = 0 fby v;` and becomes
    /// a call to `mem`.
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the node call `mem_   my_node_o_:
    /// (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        match &mut self.kind {
            stream::Kind::Expression { expression } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            stream::Kind::FollowedBy { id, constant } => {
                // add buffer to memory
                let name = symbol_table.get_name(*id);
                let typing = symbol_table.get_type(*id);
                memory.add_buffer(*id, name.clone(), typing.clone(), *constant.clone());
            }
            stream::Kind::NodeApplication {
                called_node_id,
                memory_id: node_memory_id,
                ..
            } => {
                debug_assert!(node_memory_id.is_none());
                // create fresh identifier for the new memory buffer
                let node_name = symbol_table.get_name(*called_node_id);
                let memory_name = identifier_creator.new_identifier(&node_name);
                let memory_id = symbol_table.insert_fresh_signal(memory_name, Scope::Local, None);
                memory.add_called_node(memory_id, *called_node_id);
                // put the 'memory_id' of the called node
                *node_memory_id = Some(memory_id);
            }
            stream::Kind::SomeEvent { expression } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::RisingEdge { .. } => unreachable!(),
        }
    }

    /// Change HIR expression into a normal form.
    ///
    /// The normal form of an expression is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<stream::Stmt> {
        match self.kind {
            stream::Kind::FollowedBy { ref constant, .. } => {
                // constant should already be in normal form
                debug_assert!(constant.is_normal_form());
                vec![]
            }
            stream::Kind::RisingEdge { ref mut expression } => {
                let new_statements = expression.into_signal_call(
                    nodes_reduced_graphs,
                    identifier_creator,
                    symbol_table,
                );
                if let stream::Kind::Expression {
                    expression: expr::Kind::Identifier { id },
                } = expression.kind
                {
                    let fby_dependencies = Dependencies::from(vec![(id, Label::weight(1))]);
                    let constant = stream::Expr {
                        kind: stream::Kind::expr(expr::Kind::constant(Constant::bool(
                            syn::LitBool::new(false, macro2::Span::call_site()),
                        ))),
                        typing: Some(Typ::Boolean(Default::default())),
                        location: Default::default(),
                        dependencies: Dependencies::from(vec![]),
                    };
                    let mem = stream::Expr {
                        kind: stream::Kind::fby(id, constant),
                        typing: Some(Typ::Boolean(Default::default())),
                        location: Default::default(),
                        dependencies: fby_dependencies.clone(),
                    };
                    let not_mem = stream::Expr {
                        kind: stream::Kind::expr(expr::Kind::unop(UOp::Not, mem)),
                        typing: Some(Typ::Boolean(Default::default())),
                        location: Default::default(),
                        dependencies: fby_dependencies,
                    };

                    self.dependencies =
                        Dependencies::from(vec![(id, Label::weight(0)), (id, Label::weight(1))]);
                    self.kind = stream::Kind::expr(expr::Kind::binop(
                        BOp::And,
                        *expression.clone(),
                        not_mem,
                    ));

                    new_statements
                } else {
                    unreachable!()
                }
            }
            stream::Kind::NodeApplication {
                called_node_id,
                ref mut inputs,
                ..
            } => {
                let mut new_statements = inputs
                    .iter_mut()
                    .flat_map(|(_, expression)| {
                        expression.into_signal_call(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect::<Vec<_>>();

                // change dependencies to be the sum of inputs dependencies
                let reduced_graph = nodes_reduced_graphs.get(&called_node_id).unwrap();
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(input_id, expression)| {
                            symbol_table
                                .get_node_outputs(called_node_id)
                                .iter()
                                .flat_map(|(_, output_id)| {
                                    reduced_graph.edge_weight(*output_id, *input_id).map_or(
                                        vec![],
                                        |label1| {
                                            expression
                                                .get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .map(|(id, label2)| (id, label1.add(&label2)))
                                                .collect()
                                        },
                                    )
                                })
                        })
                        .collect(),
                );

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator
                    .fresh_identifier("comp_app", symbol_table.get_name(called_node_id));
                let typing = self.get_type().cloned();
                let fresh_id =
                    symbol_table.insert_fresh_signal(fresh_name, Scope::Local, typing.clone());

                // create statement for node call
                let node_application_statement = Stmt {
                    pattern: hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Identifier { id: fresh_id },
                        typing,
                        location: self.location.clone(),
                    },
                    expression: self.clone(),
                    location: self.location.clone(),
                };
                new_statements.push(node_application_statement);

                // change current expression be an identifier to the statement of the node call
                self.kind = stream::Kind::Expression {
                    expression: expr::Kind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                new_statements
            }
            stream::Kind::Expression { ref mut expression } => expression.normal_form(
                &mut self.dependencies,
                nodes_reduced_graphs,
                identifier_creator,
                symbol_table,
            ),
            stream::Kind::SomeEvent { ref mut expression } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);
                self.dependencies = Dependencies::from(expression.get_dependencies().clone());
                new_statements
            }
            stream::Kind::NoneEvent => vec![],
        }
    }

    /// Change HIR expression into a signal call.
    ///
    /// If the expression is not a signal call, then normalize the expression,
    /// create an statement with the normalized expression and change current
    /// expression into a call to the statement.
    ///
    /// # Example
    ///
    /// The expression `1 + my_node(s, v*2).o` becomes `x` along with:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn into_signal_call(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<stream::Stmt> {
        match self.kind {
            stream::Kind::Expression {
                expression: expr::Kind::Identifier { .. },
            } => vec![],
            _ => {
                let mut statements =
                    self.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.fresh_identifier("", "x");
                let typing = self.get_type();
                let fresh_id =
                    symbol_table.insert_fresh_signal(fresh_name, Scope::Local, typing.cloned());

                // create statement for the expression
                let new_statement = Stmt {
                    pattern: hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Identifier { id: fresh_id },
                        typing: typing.cloned(),
                        location: self.location.clone(),
                    },
                    location: self.location.clone(),
                    expression: self.clone(),
                };
                statements.push(new_statement);

                // change current expression be an identifier to the statement of the expression
                self.kind = stream::Kind::Expression {
                    expression: expr::Kind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                statements
            }
        }
    }

    /// Replace identifier occurrence by element in context.
    ///
    /// It will modify the expression according to the context:
    ///
    /// - if an identifier is mapped to another identifier, then rename all occurrence of the
    ///   identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to the identifier by
    ///   the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` will become `a + b/2`.
    pub fn replace_by_context(
        &mut self,
        context_map: &HashMap<usize, Either<usize, stream::Expr>>,
    ) {
        match self.kind {
            stream::Kind::Expression { ref mut expression } => {
                let option_new_expression =
                    expression.replace_by_context(&mut self.dependencies, context_map);
                if let Some(new_expression) = option_new_expression {
                    *self = new_expression;
                }
            }
            stream::Kind::NodeApplication {
                ref mut memory_id,
                ref mut inputs,
                ..
            } => {
                // replace the id of the called node
                if let Some(element) = context_map.get(&memory_id.unwrap()) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *memory_id = Some(*new_id);
                        }
                        Either::Right(_) => unreachable!(),
                    }
                }

                inputs
                    .iter_mut()
                    .for_each(|(_, expression)| expression.replace_by_context(context_map));

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            stream::Kind::SomeEvent { ref mut expression } => {
                expression.replace_by_context(context_map);

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::FollowedBy { ref mut id, .. } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *id = *new_id;
                            self.dependencies =
                                Dependencies::from(vec![(*new_id, Label::Weight(1))]);
                        }
                        Either::Right(_) => unreachable!(),
                    }
                }
            }
            stream::Kind::RisingEdge { .. } => unreachable!(),
        }
    }
}
