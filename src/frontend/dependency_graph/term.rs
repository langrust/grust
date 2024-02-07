use crate::{
    common::graph::{color::Color, neighbor::Label, Graph},
    hir::term::{Term, TermKind},
};

impl Term {
    pub fn compute_dependencies(&self) -> Vec<String> {
        match &self.kind {
            TermKind::Binary { left, right, .. } => {
                let mut dependencies_left = left.compute_dependencies();
                let mut dependencies = right.compute_dependencies();
                dependencies.append(&mut dependencies_left);
                dependencies
            }
            TermKind::Constant { constant } => vec![],
            TermKind::Variable { signal } => vec![signal.id.clone()],
        }
    }

    pub fn add_term_dependencies(&self, node_graph: &mut Graph<Color>) {
        let mut dependencies = self.compute_dependencies();
        // signals used in the term depend on each other
        dependencies.iter().for_each(|id1| {
            dependencies.iter().for_each(|id2| {
                if id1 != id2 {
                    node_graph.add_edge(id1, id2.clone(), Label::Contract);
                    node_graph.add_edge(id2, id1.clone(), Label::Contract);
                }
            })
        })
    }
}
