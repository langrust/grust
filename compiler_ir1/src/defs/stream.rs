//! [`stream::Expr`][Expr] module.

prelude! {
    graph::{Label, DiGraphMap},
}

pub type Stmt = ir1::Stmt<Expr>;

mk_new! { impl Stmt => new {
    pattern: ir1::stmt::Pattern,
    expr: Expr,
    loc: Loc,
} }

impl Stmt {
    pub fn get_identifiers(&self) -> Vec<usize> {
        let mut identifiers = match &self.expr.kind {
            stream::Kind::Expression { expr } => match expr {
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
    /// Store component applications.
    ///
    /// # Example
    ///
    /// A statement `x: int = my_node(s, x_1).o;` increments memory with the node call
    /// `mem_my_node_o_: (my_node, o);` and the statement is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        ctx: &mut Ctx,
    ) -> Res<()> {
        self.expr.memorize(identifier_creator, memory, ctx)
    }

    /// Change [ir1] statement into a normal form.
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
        mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        ctx: &mut Ctx,
    ) -> (Vec<stream::Stmt>, Vec<stream::InitStmt>) {
        // change expression into normal form and get additional statements
        let (mut new_stmts, new_inits) = match self.expr.kind {
            stream::Kind::NodeApplication {
                called_node_id,
                ref mut inputs,
                ..
            } => {
                let (mut new_stmts, mut new_inits) = (vec![], vec![]);
                for (_, expr) in inputs.iter_mut() {
                    let (add_stmts, add_inits) =
                        expr.into_signal_call(nodes_reduced_graphs, identifier_creator, ctx);
                    new_stmts.extend(add_stmts);
                    new_inits.extend(add_inits);
                }

                // change dependencies to be the sum of inputs dependencies
                let reduced_graph = nodes_reduced_graphs.get(&called_node_id).unwrap();
                self.expr.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(input_id, expr)| {
                            ctx.get_node_outputs(called_node_id).iter().flat_map(
                                |(_, output_id)| {
                                    reduced_graph
                                        .edge_weight(*output_id, *input_id)
                                        .into_iter()
                                        .flat_map(|label1| {
                                            expr.get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .map(|(id, label2)| (id, label1.add(&label2)))
                                        })
                                },
                            )
                        })
                        .collect(),
                );

                (new_stmts, new_inits)
            }
            _ => self
                .expr
                .normal_form(nodes_reduced_graphs, identifier_creator, ctx),
        };

        // push normal_formed self statement in the statements storage
        new_stmts.push(self);

        // return statements
        (new_stmts, new_inits)
    }

    pub fn add_to_graph(&self, graph: &mut DiGraphMap<usize, Label>) {
        let signals = self.pattern.identifiers();
        for from in signals.iter() {
            for (to, label) in self.expr.get_dependencies() {
                graph::add_edge(graph, *from, *to, label.clone());
            }
        }
        match &self.expr.kind {
            stream::Kind::Expression { expr } => match expr {
                ir1::expr::Kind::Match { arms, .. } => {
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
        ctx: &mut Ctx,
    ) {
        // create fresh identifiers for the new statement
        let local_signals = self.pattern.identifiers();
        for signal_id in local_signals {
            let name = ctx.get_name(signal_id);
            let scope = ctx.get_scope(signal_id).clone();
            let fresh_name = identifier_creator.new_identifier(name.span(), &name.to_string());
            if Scope::Output != scope && &fresh_name != name {
                let typ = Some(ctx.get_typ(signal_id).clone());
                let fresh_id = ctx.insert_fresh_signal(fresh_name, scope, typ);
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
                                expr: ir1::expr::Kind::Identifier { id: new_id },
                            },
                        ..
                    }) => {
                        *signal_id = new_id.clone();
                    }
                    Either::Right(_) => noErrorDesc!(),
                }
            }
        }

        // replace identifiers in statement's expression
        new_statement.expr.replace_by_context(context_map);

        new_statement
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// component semi_fib(i: int) -> (o: int) {
    ///     init i = 1;
    ///     init aux = 0;
    ///     let aux: int = i + last i;
    ///     out o: int = last aux;
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
        ctx: &mut Ctx,
        nodes: &HashMap<usize, Component>,
    ) -> Vec<stream::Stmt> {
        let mut current_statements = vec![self.clone()];
        let mut new_statements = self.inline_when_needed(memory, identifier_creator, ctx, nodes);
        while current_statements != new_statements {
            current_statements = new_statements;
            new_statements = current_statements
                .clone()
                .into_iter()
                .flat_map(|statement| {
                    statement.inline_when_needed(memory, identifier_creator, ctx, nodes)
                })
                .collect();
        }
        new_statements
    }

    fn inline_when_needed(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        ctx: &mut Ctx,
        nodes: &HashMap<usize, Component>,
    ) -> Vec<stream::Stmt> {
        match &self.expr.kind {
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
                            ctx,
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

#[derive(Debug, Clone)]
/// Initialization statement.
pub struct InitStmt {
    /// Pattern of elements to initialize.
    pub pattern: stmt::Pattern,
    /// The expression initializing the element.
    pub expr: Expr,
    /// InitStmt location.
    pub loc: Loc,
}
impl PartialEq for InitStmt {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern && self.expr == other.expr
    }
}
impl InitStmt {
    pub fn get_identifiers(&self) -> Vec<usize> {
        let mut identifiers = match &self.expr.kind {
            stream::Kind::Expression { expr } => match expr {
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

    /// Increments memory with initializations.
    pub fn memorize(self, memory: &mut Memory, ctx: &mut Ctx) -> Res<()> {
        // add pattern to memory
        self.expr.store_in_memory(self.pattern, memory, ctx)
    }
}

pub type ExprKind = expr::Kind<Expr>;

impl ExprKind {
    /// Increment memory with expression.
    ///
    /// Store component applications.
    ///
    /// # Example
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the node call `mem_my_node_o_:
    /// (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        ctx: &mut Ctx,
    ) -> Res<()> {
        match self {
            Self::Constant { .. }
            | Self::Identifier { .. }
            | Self::Abstraction { .. }
            | Self::Enumeration { .. } => (),
            Self::UnOp { expr, .. } => {
                expr.memorize(identifier_creator, memory, ctx)?;
            }
            Self::BinOp { lft, rgt, .. } => {
                lft.memorize(identifier_creator, memory, ctx)?;
                rgt.memorize(identifier_creator, memory, ctx)?;
            }
            Self::IfThenElse { cnd, thn, els } => {
                cnd.memorize(identifier_creator, memory, ctx)?;
                thn.memorize(identifier_creator, memory, ctx)?;
                els.memorize(identifier_creator, memory, ctx)?;
            }
            Self::Application { fun, inputs } => {
                fun.memorize(identifier_creator, memory, ctx)?;
                for expr in inputs.iter_mut() {
                    expr.memorize(identifier_creator, memory, ctx)?;
                }
            }
            Self::Structure { fields, .. } => {
                for (_, expr) in fields.iter_mut() {
                    expr.memorize(identifier_creator, memory, ctx)?;
                }
            }
            Self::Array { elements } | Self::Tuple { elements } => {
                for expr in elements {
                    expr.memorize(identifier_creator, memory, ctx)?;
                }
            }
            Self::Match { expr, arms } => {
                expr.memorize(identifier_creator, memory, ctx)?;
                for (_, option, block, expr) in arms.iter_mut() {
                    if let Some(expr) = option.as_mut() {
                        expr.memorize(identifier_creator, memory, ctx)?;
                    }
                    for statement in block.iter_mut() {
                        statement.memorize(identifier_creator, memory, ctx)?;
                    }
                    expr.memorize(identifier_creator, memory, ctx)?;
                }
            }
            Self::FieldAccess { expr, .. } => {
                expr.memorize(identifier_creator, memory, ctx)?;
            }
            Self::TupleElementAccess { expr, .. } => {
                expr.memorize(identifier_creator, memory, ctx)?;
            }
            Self::Map { expr, fun } => {
                expr.memorize(identifier_creator, memory, ctx)?;
                fun.memorize(identifier_creator, memory, ctx)?;
            }
            Self::Fold { array, init, fun } => {
                array.memorize(identifier_creator, memory, ctx)?;
                init.memorize(identifier_creator, memory, ctx)?;
                fun.memorize(identifier_creator, memory, ctx)?;
            }
            Self::Sort { expr, fun } => {
                expr.memorize(identifier_creator, memory, ctx)?;
                fun.memorize(identifier_creator, memory, ctx)?;
            }
            Self::Zip { arrays } => {
                for expr in arrays {
                    expr.memorize(identifier_creator, memory, ctx)?;
                }
            }
        }
        Ok(())
    }

    /// Change [ir1] expression into a normal form.
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
        ctx: &mut Ctx,
    ) -> (Vec<stream::Stmt>, Vec<stream::InitStmt>) {
        match self {
            Self::Constant { .. }
            | Self::Identifier { .. }
            | Self::Enumeration { .. }
            | Self::Abstraction { .. } => (vec![], vec![]),
            Self::UnOp { expr, .. } => {
                let (new_stmts, new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);

                *dependencies = Dependencies::from(expr.get_dependencies().clone());

                (new_stmts, new_inits)
            }

            Self::BinOp { lft, rgt, .. } => {
                let (mut new_stmts, mut new_inits) =
                    lft.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                let (stmts, inits) = rgt.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                new_stmts.extend(stmts);
                new_inits.extend(inits);

                let mut expression_dependencies = lft.get_dependencies().clone();
                let mut other_dependencies = rgt.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);

                (new_stmts, new_inits)
            }

            Self::IfThenElse { cnd, thn, els } => {
                let (mut new_stmts, mut new_inits) =
                    cnd.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                let (stmts, inits) = thn.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                new_stmts.extend(stmts);
                new_inits.extend(inits);
                let (stmts, inits) = els.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                new_stmts.extend(stmts);
                new_inits.extend(inits);

                let mut expr_dependencies = cnd.get_dependencies().clone();
                let mut other_dependencies = thn.get_dependencies().clone();
                expr_dependencies.append(&mut other_dependencies);
                let mut other_dependencies = els.get_dependencies().clone();
                expr_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expr_dependencies);

                (new_stmts, new_inits)
            }

            Self::Application { ref mut inputs, .. } => {
                let (mut new_stmts, mut new_inits) = (vec![], vec![]);
                for expr in inputs.iter_mut() {
                    let (add_stmts, add_inits) =
                        expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                    new_stmts.extend(add_stmts);
                    new_inits.extend(add_inits);
                }

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expr| expr.get_dependencies().clone())
                        .collect(),
                );

                (new_stmts, new_inits)
            }

            Self::Structure { fields, .. } => {
                let (mut new_stmts, mut new_inits) = (vec![], vec![]);
                for (_, expr) in fields.iter_mut() {
                    let (add_stmts, add_inits) =
                        expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                    new_stmts.extend(add_stmts);
                    new_inits.extend(add_inits);
                }

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );

                (new_stmts, new_inits)
            }
            Self::Array { elements } | Self::Tuple { elements } => {
                let (mut new_stmts, mut new_inits) = (vec![], vec![]);
                for expr in elements.iter_mut() {
                    let (add_stmts, add_inits) =
                        expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                    new_stmts.extend(add_stmts);
                    new_inits.extend(add_inits);
                }

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );

                (new_stmts, new_inits)
            }
            Self::Match { expr, arms, .. } => {
                let (mut new_stmts, mut new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                let mut expr_dependencies = expr.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(pattern, bound, body, matched_expr)| {
                        // get local signals defined in pattern
                        let local_signals = pattern.identifiers();

                        // normalize body statements
                        *body = body
                            .iter_mut()
                            .flat_map(|statement| {
                                let (stmts, inits) = statement.clone().normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    ctx,
                                );
                                debug_assert!(inits.is_empty());
                                stmts
                            })
                            .collect();

                        // remove identifiers created by the pattern from the dependencies
                        bound.as_mut().map(|expr| {
                            let (stmts, inits) =
                                expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                            new_stmts.extend(stmts);
                            new_inits.extend(inits);

                            expr_dependencies.extend(
                                expr.get_dependencies()
                                    .clone()
                                    .into_iter()
                                    .filter(|(signal, _)| !local_signals.contains(signal)),
                            )
                        });

                        let (stmts, inits) =
                            matched_expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                        body.extend(stmts);
                        debug_assert!(inits.is_empty());

                        expr_dependencies.extend(
                            matched_expr
                                .get_dependencies()
                                .clone()
                                .into_iter()
                                .filter(|(signal, _)| !local_signals.contains(signal)),
                        );
                    });

                *dependencies = Dependencies::from(expr_dependencies);

                (new_stmts, new_inits)
            }
            Self::FieldAccess { expr, .. } => {
                let (new_stmts, new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);

                *dependencies = Dependencies::from(expr.get_dependencies().clone());

                (new_stmts, new_inits)
            }
            Self::TupleElementAccess { expr, .. } => {
                let (new_stmts, new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);

                *dependencies = Dependencies::from(expr.get_dependencies().clone());

                (new_stmts, new_inits)
            }
            Self::Map { expr, .. } => {
                let (new_stmts, new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);

                *dependencies = Dependencies::from(expr.get_dependencies().clone());

                (new_stmts, new_inits)
            }
            Self::Fold { array, init, .. } => {
                let (mut new_stmts, mut new_inits) =
                    array.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                let (stmts, inits) =
                    init.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                new_stmts.extend(stmts);
                new_inits.extend(inits);

                // get matched expressions dependencies
                let mut expr_dependencies = array.get_dependencies().clone();
                let mut init_dependencies = array.get_dependencies().clone();
                expr_dependencies.append(&mut init_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expr_dependencies);

                (new_stmts, new_inits)
            }
            Self::Sort { expr, .. } => {
                let (new_stmts, new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);

                *dependencies = Dependencies::from(expr.get_dependencies().clone());

                (new_stmts, new_inits)
            }
            Self::Zip { arrays, .. } => {
                let (mut new_stmts, mut new_inits) = (vec![], vec![]);
                for expr in arrays.iter_mut() {
                    let (add_stmts, add_inits) =
                        expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                    new_stmts.extend(add_stmts);
                    new_inits.extend(add_inits);
                }

                *dependencies = Dependencies::from(
                    arrays
                        .iter()
                        .flat_map(|array| array.get_dependencies().clone())
                        .collect(),
                );

                (new_stmts, new_inits)
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
            Self::UnOp { expr, .. } => {
                expr.replace_by_context(context_map);
                *dependencies = Dependencies::from(expr.get_dependencies().clone());
                None
            }
            Self::BinOp { lft, rgt, .. } => {
                lft.replace_by_context(context_map);
                rgt.replace_by_context(context_map);

                let mut expression_dependencies = lft.get_dependencies().clone();
                let mut other_dependencies = rgt.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::IfThenElse { cnd, thn, els } => {
                cnd.replace_by_context(context_map);
                thn.replace_by_context(context_map);
                els.replace_by_context(context_map);

                let mut expr_dependencies = cnd.get_dependencies().clone();
                let mut other_dependencies = thn.get_dependencies().clone();
                expr_dependencies.append(&mut other_dependencies);
                let mut other_dependencies = els.get_dependencies().clone();
                expr_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expr_dependencies);
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
                ref mut expr,
                ref mut arms,
                ..
            } => {
                expr.replace_by_context(context_map);
                let mut expr_dependencies = expr.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(pattern, bound, body, matched_expr)| {
                        let local_signals = pattern.identifiers();

                        // remove identifiers created by the pattern from the context
                        let context_map = context_map
                            .clone()
                            .into_iter()
                            .filter(|(key, _)| !local_signals.contains(key))
                            .collect();

                        if let Some(expr) = bound.as_mut() {
                            expr.replace_by_context(&context_map);
                            let mut bound_dependencies = expr
                                .get_dependencies()
                                .clone()
                                .into_iter()
                                .filter(|(signal, _)| !local_signals.contains(signal))
                                .collect();
                            expr_dependencies.append(&mut bound_dependencies);
                        };

                        body.iter_mut()
                            .for_each(|statement| statement.expr.replace_by_context(&context_map));

                        matched_expr.replace_by_context(&context_map);
                        let matched_expr_dependencies = matched_expr
                            .get_dependencies()
                            .clone()
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal));
                        expr_dependencies.extend(matched_expr_dependencies);
                    });

                *dependencies = Dependencies::from(expr_dependencies);
                None
            }
            Self::FieldAccess { ref mut expr, .. } => {
                expr.replace_by_context(context_map);
                // get matched expression dependencies
                let expr_dependencies = expr.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expr_dependencies);
                None
            }
            Self::TupleElementAccess { ref mut expr, .. } => {
                expr.replace_by_context(context_map);
                // get matched expression dependencies
                let expr_dependencies = expr.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expr_dependencies);
                None
            }
            Self::Map { ref mut expr, .. } => {
                expr.replace_by_context(context_map);
                // get matched expression dependencies
                let expr_dependencies = expr.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expr_dependencies);
                None
            }
            Self::Fold {
                ref mut array,
                ref mut init,
                ..
            } => {
                array.replace_by_context(context_map);
                init.replace_by_context(context_map);
                // get matched expressions dependencies
                let mut expr_dependencies = array.get_dependencies().clone();
                let mut init_dependencies = array.get_dependencies().clone();
                expr_dependencies.append(&mut init_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expr_dependencies);
                None
            }
            Self::Sort { ref mut expr, .. } => {
                expr.replace_by_context(context_map);
                // get matched expression dependencies
                let expr_dependencies = expr.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expr_dependencies);
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
        expr: expr::Kind<Expr>,
    },
    /// Initialized buffer stream expression.
    Last {
        /// The initialization id.
        init_id: usize,
        /// The id of signal that is initialized.
        signal_id: usize,
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
        expr: Box<Expr>,
    },
    /// Present event expression.
    SomeEvent {
        /// The expression of the event.
        expr: Box<Expr>,
    },
    /// Absent event expression.
    NoneEvent,
}

impl Kind {
    pub fn is_default_constant(&self) -> bool {
        if let Self::Expression { expr } = self {
            expr.is_default_constant()
        } else {
            false
        }
    }
}

mk_new! { impl Kind =>
    Expression: expr {
        expr: impl Into<expr::Kind<Expr>> = expr.into(),
    }
    Last: last { init_id: usize, signal_id: usize }
    NodeApplication: call {
        memory_id = None,
        called_node_id: usize,
        inputs: Vec<(usize, Expr)>,
    }
    RisingEdge: rising_edge {
        expr: Expr = expr.into(),
    }
    SomeEvent: some_event {
        expr: Expr = expr.into(),
    }
    NoneEvent: none_event ()
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression AST.
pub struct Expr {
    /// Stream expression kind.
    pub kind: Kind,
    /// Stream expression type.
    pub typ: Option<Typ>,
    /// Stream expression location.
    pub loc: Loc,
    /// Stream expression dependencies.
    pub dependencies: Dependencies,
}
impl HasLoc for Expr {
    fn loc(&self) -> Loc {
        self.loc
    }
}

impl Expr {
    /// Constructs stream expression.
    ///
    /// Typing, location and dependencies are empty.
    pub fn new(loc: impl Into<Loc>, kind: Kind) -> Expr {
        Expr {
            kind,
            typ: None,
            loc: loc.into(),
            dependencies: Dependencies::new(),
        }
    }

    pub fn is_default_constant(&self) -> bool {
        self.kind.is_default_constant()
    }

    /// Get stream expression's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typ.as_ref()
    }
    /// Get stream expression's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typ.as_mut()
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
        let predicate_expr = |expr: &Expr| expr.no_component_application() && expr.no_rising_edge();
        let predicate_statement = |statement: &Stmt| statement.expr.is_normal_form();
        match &self.kind {
            Kind::Expression { expr } => {
                expr.propagate_predicate(predicate_expr, predicate_statement)
            }
            Kind::Last { .. } => true,
            Kind::NodeApplication { inputs, .. } => {
                inputs.iter().all(|(_, expr)| predicate_expr(expr))
            }
            Kind::SomeEvent { expr } => predicate_expr(expr),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => false,
        }
    }

    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        match &self.kind {
            Kind::Expression { expr } => expr
                .propagate_predicate(Self::no_component_application, |statement| {
                    statement.expr.no_component_application()
                }),
            Kind::Last { .. } => true,
            Kind::NodeApplication { .. } => false,
            Kind::SomeEvent { expr } => expr.no_component_application(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { expr } => expr.no_component_application(),
        }
    }

    /// Tell if there is no rising edge.
    pub fn no_rising_edge(&self) -> bool {
        match &self.kind {
            Kind::Expression { expr } => expr
                .propagate_predicate(Self::no_rising_edge, |statement| {
                    statement.expr.no_rising_edge()
                }),
            Kind::Last { .. } => true,
            Kind::NodeApplication { inputs, .. } => {
                inputs.iter().all(|(_, expr)| expr.no_rising_edge())
            }
            Kind::SomeEvent { expr } => expr.no_rising_edge(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => false,
        }
    }

    /// Increment memory with expression.
    ///
    /// Store component applications.
    ///
    /// # Example
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the node call `mem_   my_node_o_:
    /// (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        ctx: &mut Ctx,
    ) -> Res<()> {
        match &mut self.kind {
            stream::Kind::Expression { expr } => {
                expr.memorize(identifier_creator, memory, ctx)?;
            }
            stream::Kind::Last { .. } => (),
            stream::Kind::NodeApplication {
                called_node_id,
                memory_id: node_memory_id,
                ..
            } => {
                debug_assert!(node_memory_id.is_none());
                // create fresh identifier for the new memory buffer
                let node_name = ctx.get_name(*called_node_id);
                let memory_name =
                    identifier_creator.new_identifier(node_name.loc(), &node_name.to_string());
                let memory_id = ctx.insert_fresh_signal(memory_name, Scope::Local, None);
                memory.add_called_node(memory_id, *called_node_id);
                // put the 'memory_id' of the called node
                *node_memory_id = Some(memory_id);
            }
            stream::Kind::SomeEvent { expr } => {
                expr.memorize(identifier_creator, memory, ctx)?;
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::RisingEdge { .. } => noErrorDesc!(),
        }
        Ok(())
    }

    pub fn store_in_memory(
        mut self,
        mut pat: stmt::Pattern,
        memory: &mut Memory,
        ctx: &mut Ctx,
    ) -> Res<()> {
        match (self.kind, pat.kind) {
            (
                Kind::Expression {
                    expr:
                        expr::Kind::Tuple {
                            elements: expr_elem,
                        },
                },
                stmt::Kind::Tuple {
                    elements: pat_elems,
                },
            ) => {
                res_vec!(expr_elem
                    .into_iter()
                    .zip(pat_elems.into_iter())
                    .map(|(expr, pat)| expr.store_in_memory(pat, memory, ctx)));
                Ok(())
            }
            (a, b) => {
                self.kind = a;
                pat.kind = b;
                let id = pat
                    .identifiers()
                    .pop()
                    .expect("internal error: should be checked at typing");
                // add buffer to memory
                let name = ctx.get_name(id);
                let typ = ctx.get_typ(id);
                memory.add_buffer(id, name.clone(), typ.clone(), self)
            }
        }
    }

    /// Change [ir1] expression into a normal form.
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
        ctx: &mut Ctx,
    ) -> (Vec<stream::Stmt>, Vec<stream::InitStmt>) {
        let loc = self.loc;
        match self.kind {
            stream::Kind::Last { .. } => (vec![], vec![]),
            stream::Kind::RisingEdge { ref mut expr } => {
                let (new_stmts, mut new_inits) =
                    expr.into_signal_call(nodes_reduced_graphs, identifier_creator, ctx);
                if let stream::Kind::Expression {
                    expr: expr::Kind::Identifier { id },
                } = expr.kind
                {
                    let last_dependencies = Dependencies::from(vec![(id, Label::weight(1))]);
                    let init_pat = stmt::Pattern {
                        kind: stmt::Kind::ident(id),
                        typ: Some(Typ::Boolean(Default::default())),
                        loc,
                    };
                    let init_expr = stream::Expr {
                        kind: stream::Kind::expr(expr::Kind::constant(Constant::bool(
                            syn::LitBool::new(false, macro2::Span::call_site()),
                        ))),
                        typ: Some(Typ::Boolean(Default::default())),
                        loc,
                        dependencies: Dependencies::from(vec![]),
                    };
                    let init_mem = stream::InitStmt {
                        pattern: init_pat,
                        expr: init_expr,
                        loc,
                    };
                    new_inits.push(init_mem);

                    let mem = stream::Expr {
                        kind: stream::Kind::last(id, id), // todo: is it ok?
                        typ: Some(Typ::Boolean(Default::default())),
                        loc,
                        dependencies: last_dependencies.clone(),
                    };
                    let not_mem = stream::Expr {
                        kind: stream::Kind::expr(expr::Kind::unop(UOp::Not, mem)),
                        typ: Some(Typ::Boolean(Default::default())),
                        loc,
                        dependencies: last_dependencies,
                    };

                    self.dependencies =
                        Dependencies::from(vec![(id, Label::weight(0)), (id, Label::weight(1))]);
                    self.kind =
                        stream::Kind::expr(expr::Kind::binop(BOp::And, *expr.clone(), not_mem));

                    (new_stmts, new_inits)
                } else {
                    noErrorDesc!("internal error: rising edge should be detected on ident only")
                }
            }
            stream::Kind::NodeApplication {
                called_node_id,
                ref mut inputs,
                ..
            } => {
                let (mut new_stmts, mut new_inits) = (vec![], vec![]);
                for (_, expr) in inputs.iter_mut() {
                    let (add_stmts, add_inits) =
                        expr.into_signal_call(nodes_reduced_graphs, identifier_creator, ctx);
                    new_stmts.extend(add_stmts);
                    new_inits.extend(add_inits);
                }

                // change dependencies to be the sum of inputs dependencies
                let reduced_graph = nodes_reduced_graphs.get(&called_node_id).unwrap();
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(input_id, expr)| {
                            ctx.get_node_outputs(called_node_id).iter().flat_map(
                                |(_, output_id)| {
                                    reduced_graph
                                        .edge_weight(*output_id, *input_id)
                                        .into_iter()
                                        .flat_map(|label1| {
                                            expr.get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .map(|(id, label2)| (id, label1.add(&label2)))
                                        })
                                },
                            )
                        })
                        .collect(),
                );

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.fresh_identifier(
                    loc,
                    "comp_app",
                    &ctx.get_name(called_node_id).to_string(),
                );
                let typ = self.get_type().cloned();
                let fresh_id = ctx.insert_fresh_signal(fresh_name, Scope::Local, typ.clone());

                // create statement for node call
                let node_application_statement = Stmt {
                    pattern: ir1::stmt::Pattern {
                        kind: ir1::stmt::Kind::Identifier { id: fresh_id },
                        typ,
                        loc: self.loc.clone(),
                    },
                    expr: self.clone(),
                    loc: self.loc.clone(),
                };
                new_stmts.push(node_application_statement);

                // change current expression be an identifier to the statement of the node call
                self.kind = stream::Kind::Expression {
                    expr: expr::Kind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                (new_stmts, new_inits)
            }
            stream::Kind::Expression { ref mut expr } => expr.normal_form(
                &mut self.dependencies,
                nodes_reduced_graphs,
                identifier_creator,
                ctx,
            ),
            stream::Kind::SomeEvent { ref mut expr } => {
                let (new_stmts, new_inits) =
                    expr.normal_form(nodes_reduced_graphs, identifier_creator, ctx);
                self.dependencies = Dependencies::from(expr.get_dependencies().clone());
                (new_stmts, new_inits)
            }
            stream::Kind::NoneEvent => (vec![], vec![]),
        }
    }

    /// Change [ir1] expression into a signal call.
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
        ctx: &mut Ctx,
    ) -> (Vec<stream::Stmt>, Vec<stream::InitStmt>) {
        match self.kind {
            stream::Kind::Expression {
                expr: expr::Kind::Identifier { .. },
            } => (vec![], vec![]),
            _ => {
                let (mut new_stmts, new_inits) =
                    self.normal_form(nodes_reduced_graphs, identifier_creator, ctx);

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.fresh_identifier(self.loc(), "", "x");
                let typ = self.get_type();
                let fresh_id = ctx.insert_fresh_signal(fresh_name, Scope::Local, typ.cloned());

                // create statement for the expression
                let new_statement = Stmt {
                    pattern: ir1::stmt::Pattern {
                        kind: ir1::stmt::Kind::Identifier { id: fresh_id },
                        typ: typ.cloned(),
                        loc: self.loc.clone(),
                    },
                    loc: self.loc.clone(),
                    expr: self.clone(),
                };
                new_stmts.push(new_statement);

                // change current expression be an identifier to the statement of the expression
                self.kind = stream::Kind::Expression {
                    expr: expr::Kind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                (new_stmts, new_inits)
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
            stream::Kind::Expression { ref mut expr } => {
                let option_new_expr = expr.replace_by_context(&mut self.dependencies, context_map);
                if let Some(new_expr) = option_new_expr {
                    *self = new_expr;
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
                                    expr: ir1::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *memory_id = Some(*new_id);
                        }
                        Either::Right(_) => noErrorDesc!(),
                    }
                }

                inputs
                    .iter_mut()
                    .for_each(|(_, expr)| expr.replace_by_context(context_map));

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expr)| expr.get_dependencies().clone())
                        .collect(),
                );
            }
            stream::Kind::SomeEvent { ref mut expr } => {
                expr.replace_by_context(context_map);

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(expr.get_dependencies().clone());
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::Last {
                ref mut signal_id,
                ref mut init_id,
            } => {
                if let Some(element) = context_map.get(signal_id) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expr: ir1::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *signal_id = *new_id;
                            self.dependencies =
                                Dependencies::from(vec![(*new_id, Label::Weight(1))]);
                        }
                        Either::Right(_) => noErrorDesc!(),
                    }
                }
                if let Some(element) = context_map.get(init_id) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expr: ir1::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *init_id = *new_id;
                        }
                        Either::Right(_) => noErrorDesc!(),
                    }
                }
            }
            stream::Kind::RisingEdge { .. } => noErrorDesc!(),
        }
    }
}
