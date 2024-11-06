prelude! {
    graph::*,
    hir::{ Dependencies, IdentifierCreator, stream },
}

impl hir::expr::Kind<stream::Expr> {
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
            Self::Unop { expression, .. } => {
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
}
