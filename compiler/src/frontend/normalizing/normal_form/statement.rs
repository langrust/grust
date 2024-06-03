prelude! {
    graph::*,
    hir::{ Dependencies, IdentifierCreator, Stmt, stream },
}

impl Stmt<stream::Expr> {
    /// Change HIR statement into a normal form.
    ///
    /// The normal form of an statement is as follows:
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
    ) -> Vec<Stmt<stream::Expr>> {
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
        for from in signals {
            for (to, label) in expression.get_dependencies() {
                graph.add_edge(from, *to, label.clone());
            }
        }
        match &self.expression.kind {
            stream::Kind::Expression { expression } => match expression {
                hir::expr::Kind::Match { arms, .. } => {
                    arms.iter().for_each(|(_, _, statements, _)| {
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
}
