use petgraph::graphmap::DiGraphMap;

use crate::{
    ast::interface::FlowKind,
    hir::{
        flow_expression::{FlowExpression, FlowExpressionKind},
        interface::{FlowDeclaration, FlowInstanciation, FlowStatement, Interface},
    },
    symbol_table::SymbolTable,
};

impl Interface {
    /// Compute the dependency graph of the interface.
    ///
    /// # Example
    ///
    /// ```GR
    /// import signal s;
    /// import event e;
    /// export signal o;
    ///
    /// let event e2 = timeout(e, 30);  // depends on e
    /// o = my_component(s, e2);        // depends on s and e2
    /// ```
    pub fn compute_dependencies(&mut self, symbol_table: &SymbolTable) {
        // initiate graph
        let mut graph = self.create_initialized_graph();

        // complete dependency graphs
        self.statements
            .iter()
            .for_each(|statement| statement.add_dependencies(&mut graph, symbol_table));

        // set interface's graph
        self.graph = graph;
    }

    /// Create an initialized graph from an interface.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    fn create_initialized_graph(&self) -> DiGraphMap<usize, FlowKind> {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // add flows as vertices
        for flow in self.get_flows_ids() {
            graph.add_node(flow);
        }

        // return graph
        graph
    }
}

impl FlowStatement {
    pub fn add_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, FlowKind>,
        symbol_table: &SymbolTable,
    ) {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                pattern,
                flow_expression,
                ..
            })
            | FlowStatement::Instanciation(FlowInstanciation {
                pattern,
                flow_expression,
                ..
            }) => {
                let flows_ids = pattern.identifiers();
                let dependencies = flow_expression.get_dependencies();
                for flow_id in flows_ids {
                    dependencies.iter().for_each(|source_id| {
                        graph.add_edge(*source_id, flow_id, symbol_table.get_flow_kind(*source_id));
                    });
                }
            }
            FlowStatement::Import(_) | FlowStatement::Export(_) => (),
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
