use petgraph::graphmap::DiGraphMap;

use crate::{
    common::label::Label,
    hir::contract::{Term, TermKind},
};

use super::add_edge;

impl Term {
    /// Compute dependencies of a term.
    pub fn compute_dependencies(&self) -> Vec<usize> {
        match &self.kind {
            TermKind::Unary { term, .. } => term.compute_dependencies(),
            TermKind::Binary { left, right, .. } => {
                let mut dependencies_left = left.compute_dependencies();
                let mut dependencies = right.compute_dependencies();
                dependencies.append(&mut dependencies_left);
                dependencies
            }
            TermKind::Constant { .. } => vec![],
            TermKind::Identifier { id } => vec![*id],
        }
    }

    /// Add dependencies of a term to the graph.
    pub fn add_term_dependencies(&self, node_graph: &mut DiGraphMap<usize, Label>) {
        let dependencies = self.compute_dependencies();
        // signals used in the term depend on each other
        dependencies.iter().for_each(|id1| {
            dependencies.iter().for_each(|id2| {
                if id1 != id2 {
                    add_edge(node_graph, *id1, *id2, Label::Contract);
                    add_edge(node_graph, *id2, *id1, Label::Contract);
                }
            })
        })
    }
}
