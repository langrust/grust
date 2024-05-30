use petgraph::graphmap::DiGraphMap;

prelude! {
    common::HashMap,
    hir::{
        flow_expression::{FlowExpression, FlowExpressionKind},
        interface::{
            FlowDeclaration, FlowExport, FlowImport, FlowInstanciation, FlowStatement, Interface,
        },
    },
}

impl Interface {
    /// Compute the dependency graph of the interface.
    ///
    /// # Example
    ///
    /// ```GR
    /// statement 1) import signal s;
    /// statement 2) import event e;
    /// statement 3) export signal o;                // depends on statement 5)
    ///
    /// statement 4) let event e2 = timeout(e, 30);  // depends on statement 2)
    /// statement 5) o = my_component(s, e2);        // depends on statement 1) and 4)
    /// ```
    pub fn compute_dependencies(&mut self) {
        // initiate graph
        let mut graph = self.create_initialized_graph();

        // create map from flow id to statement defining the flow
        let flows_statements = self.create_map_from_flow_id_to_statement_id();

        // complete dependency graphs
        self.statements
            .iter()
            .enumerate()
            .for_each(|(index, statement)| {
                statement.add_dependencies(index, &flows_statements, &mut graph)
            });

        // set interface's graph
        self.graph = graph;
    }

    fn create_map_from_flow_id_to_statement_id(&self) -> HashMap<usize, usize> {
        let mut flows_statements = common::new_hmap();
        self.statements
            .iter()
            .enumerate()
            .for_each(|(index, statement)| {
                match &statement {
                    FlowStatement::Import(FlowImport { id, .. }) => {
                        flows_statements.insert(*id, index);
                    }
                    FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                    | FlowStatement::Instanciation(FlowInstanciation { pattern, .. }) => {
                        pattern.identifiers().into_iter().for_each(|id| {
                            flows_statements.insert(id, index);
                        });
                    }
                    FlowStatement::Export(_) => (), // flows are computed by the instanciate statement
                };
            });
        flows_statements
    }

    /// Create an initialized graph from an interface.
    ///
    /// The created graph has every statements' indexes as vertices.
    /// But no edges are added.
    fn create_initialized_graph(&self) -> DiGraphMap<usize, ()> {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // add flows as vertices
        for statement_index in 0..self.statements.len() {
            graph.add_node(statement_index);
        }

        // return graph
        graph
    }
}

impl FlowStatement {
    pub fn add_dependencies(
        &self,
        index: usize,
        flows_statements: &HashMap<usize, usize>,
        graph: &mut DiGraphMap<usize, ()>,
    ) {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                flow_expression, ..
            })
            | FlowStatement::Instanciation(FlowInstanciation {
                flow_expression, ..
            }) => {
                let dependencies = flow_expression.get_dependencies();
                dependencies.iter().for_each(|flow_id| {
                    let index_statement = flows_statements.get(flow_id).expect("should be there");
                    graph.add_edge(*index_statement, index, ());
                });
            }
            FlowStatement::Export(FlowExport { id, .. }) => {
                let index_statement = flows_statements.get(id).expect("should be there");
                graph.add_edge(*index_statement, index, ());
            }
            FlowStatement::Import(_) => (), // no dependencies for import
        }
    }
}

impl FlowExpression {
    pub fn get_dependencies(&self) -> Vec<usize> {
        match &self.kind {
            FlowExpressionKind::Ident { id } => vec![*id],
            FlowExpressionKind::Sample {
                flow_expression, ..
            }
            | FlowExpressionKind::Scan {
                flow_expression, ..
            }
            | FlowExpressionKind::Timeout {
                flow_expression, ..
            }
            | FlowExpressionKind::Throtle {
                flow_expression, ..
            }
            | FlowExpressionKind::OnChange { flow_expression } => {
                flow_expression.get_dependencies()
            }
            FlowExpressionKind::ComponentCall { inputs, .. } => inputs
                .iter()
                .flat_map(|(_, flow_expression)| flow_expression.get_dependencies())
                .collect(),
        }
    }
}
