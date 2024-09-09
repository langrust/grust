prelude! {
    graph::*,
    hir::{ Dependencies, IdentifierCreator, Stmt, stream },
}

impl stream::Expr {
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
    ) -> Vec<Stmt<stream::Expr>> {
        match self.kind {
            stream::Kind::FollowedBy {
                ref mut expression,
                ref constant,
            } => {
                // constant should already be in normal form
                debug_assert!(constant.is_normal_form());

                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                self.dependencies = Dependencies::from(
                    expression
                        .get_dependencies()
                        .iter()
                        .map(|(id, label)| (id.clone(), label.increment()))
                        .collect(),
                );

                new_statements
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
                    expression: hir::expr::Kind::Identifier { id: fresh_id },
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
    ) -> Vec<Stmt<stream::Expr>> {
        match self.kind {
            stream::Kind::Expression {
                expression: hir::expr::Kind::Identifier { .. },
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
                    expression: hir::expr::Kind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                statements
            }
        }
    }
}
