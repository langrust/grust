prelude! {
    graph::DiGraphMap,
    hir::{
        flow, interface::{ FlowDeclaration, FlowInstantiation, FlowStatement, Interface, Service }
    },
}

impl Interface {
    /// Generate dependency graphs for every services.
    #[inline]
    pub fn generate_flows_dependency_graphs(&mut self) {
        let flows_imports = self.flow_id_to_import_id();
        let flows_exports = self.flow_id_to_export_id();
        self.services
            .iter_mut()
            .for_each(|service| service.compute_dependencies(&flows_imports, &flows_exports))
    }

    /// Create a map from flow identifier to its import statement.
    fn flow_id_to_import_id(&self) -> HashMap<usize, usize> {
        let mut flows_imports = HashMap::new();
        self.imports.iter().for_each(|(stmt_id, import)| {
            flows_imports.insert(import.id, *stmt_id);
        });
        flows_imports
    }

    /// Create a map from flow identifier to its export statement.
    fn flow_id_to_export_id(&self) -> HashMap<usize, usize> {
        let mut flows_exports = HashMap::new();
        self.exports.iter().for_each(|(stmt_id, export)| {
            flows_exports.insert(export.id, *stmt_id);
        });
        flows_exports
    }
}

impl Service {
    /// Compute the dependency graph of the service.
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
    pub fn compute_dependencies(
        &mut self,
        flows_imports: &HashMap<usize, usize>,
        flows_exports: &HashMap<usize, usize>,
    ) {
        // create map from flow id to statement defining the flow
        let flows_statements = self.flow_id_to_statement_id(flows_imports);

        // initiate graph
        let mut graph = self.create_initialized_graph(&flows_statements, flows_exports);

        // complete dependency graphs
        self.statements.iter().for_each(|(stmt_id, statement)| {
            statement.add_dependencies(*stmt_id, &flows_statements, &mut graph)
        });

        // set service's graph
        self.graph = graph;
    }

    /// Create a map from flow identifier to its definition statement.
    ///
    /// Export statements do not define the flow, it is the instanciation statement that defines it.
    /// Then, when a flow is exported, it is the instanciation statement that is linked to the flow
    /// identifier in the map.
    fn flow_id_to_statement_id(
        &self,
        flows_imports: &HashMap<usize, usize>,
    ) -> HashMap<usize, usize> {
        let mut flows_statements = flows_imports.clone();

        self.statements.iter().for_each(|(stmt_id, statement)| {
            match &statement {
                FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                | FlowStatement::Instantiation(FlowInstantiation { pattern, .. }) => {
                    pattern.identifiers().into_iter().for_each(|id| {
                        flows_statements.insert(id, *stmt_id);
                    });
                }
            };
        });

        flows_statements
    }

    /// Create an initialized graph from a service.
    ///
    /// The created graph has every statements' indexes as vertices.
    /// But no edges are added.
    fn create_initialized_graph(
        &self,
        flows_statements: &HashMap<usize, usize>,
        flows_exports: &HashMap<usize, usize>,
    ) -> DiGraphMap<usize, ()> {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // add service's statements as vertices
        for stmt_id in self.statements.keys() {
            graph.add_node(*stmt_id);
        }

        // add potential dependencies between export and service's statements
        flows_exports.iter().for_each(|(flow_id, export_id)| {
            if let Some(stmt_id) = flows_statements.get(flow_id) {
                graph.add_edge(*stmt_id, *export_id, ());
            }
        });

        // return graph
        graph
    }
}

impl FlowStatement {
    pub fn add_dependencies(
        &self,
        stmt_id: usize,
        flows_statements: &HashMap<usize, usize>,
        graph: &mut DiGraphMap<usize, ()>,
    ) {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                flow_expression, ..
            })
            | FlowStatement::Instantiation(FlowInstantiation {
                flow_expression, ..
            }) => {
                let dependencies = flow_expression.get_dependencies();
                dependencies.iter().for_each(|flow_id| {
                    let index_statement = flows_statements.get(flow_id).expect("should be there");
                    graph.add_edge(*index_statement, stmt_id, ());
                });
            }
        }
    }
}

impl flow::Expr {
    pub fn get_dependencies(&self) -> Vec<usize> {
        match &self.kind {
            flow::Kind::Ident { id } => vec![*id],
            flow::Kind::Sample {
                flow_expression, ..
            }
            | flow::Kind::Scan {
                flow_expression, ..
            }
            | flow::Kind::Timeout {
                flow_expression, ..
            }
            | flow::Kind::Throttle {
                flow_expression, ..
            }
            | flow::Kind::OnChange { flow_expression } => flow_expression.get_dependencies(),
            flow::Kind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => {
                let mut dependencies = flow_expression_1.get_dependencies();
                dependencies.extend(flow_expression_2.get_dependencies());
                dependencies
            }
            flow::Kind::ComponentCall { inputs, .. } => inputs
                .iter()
                .flat_map(|(_, flow_expression)| flow_expression.get_dependencies())
                .collect(),
        }
    }
}
