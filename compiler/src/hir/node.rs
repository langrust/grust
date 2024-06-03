//! HIR [Node](crate::hir::node::Node) module.

prelude! {
    graph::*,
    hir::{Contract, Stmt, stream},
}

use super::memory::Memory;

#[derive(Debug, Clone)]
/// LanGRust node HIR.
pub struct Node {
    /// Node identifier.
    pub id: usize,
    /// Node's statements.
    pub statements: Vec<Stmt<stream::Expr>>,
    /// Node's contract.
    pub contract: Contract,
    /// Node location.
    pub location: Location,
    /// Node dependency graph.
    pub graph: DiGraphMap<usize, Label>,
    /// Unitary node's memory.
    pub memory: Memory,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.statements == other.statements
            && self.contract == other.contract
            && self.location == other.location
            && self.eq_graph(other)
    }
}

impl Node {
    fn eq_graph(&self, other: &Node) -> bool {
        let graph_nodes = self.graph.nodes();
        let other_nodes = other.graph.nodes();
        let graph_edges = self.graph.all_edges();
        let other_edges = other.graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }

    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.statements.iter().all(|statement| statement.no_fby())
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.is_normal_form())
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.no_node_application())
    }
}
