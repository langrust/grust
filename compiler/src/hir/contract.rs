//! HIR [Contract](crate::hir::contract::Contract) module.

prelude! {
    operator::{BinaryOperator, UnaryOperator},
    graph::*,
}

#[derive(Debug, PartialEq, Clone)]
/// Term's kind.
pub enum TermKind {
    /// Constant term: 3
    Constant {
        /// The constant
        constant: Constant,
    },
    /// Identifier term: x
    Identifier {
        /// Signal's identifier in Symbol Table.
        id: usize,
    },
    /// Unary term: !x
    Unary {
        /// The operator
        op: UnaryOperator,
        /// The term
        term: Box<Term>,
    },
    /// Binary term: x == y
    Binary {
        /// The operator
        op: BinaryOperator,
        /// Left term
        left: Box<Term>,
        /// Right term
        right: Box<Term>,
    },
}

#[derive(Debug, PartialEq, Clone)]
/// Contract's term.
pub struct Term {
    /// The kind of the term
    pub kind: TermKind,
    /// The location in source code
    pub location: Location,
}

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

#[derive(Debug, Default, PartialEq, Clone)]
/// Contract to prove using Creusot.
pub struct Contract {
    /// Requirements clauses to suppose
    pub requires: Vec<Term>,
    /// Ensures clauses to prove
    pub ensures: Vec<Term>,
    /// Invariant clauses to prove
    pub invariant: Vec<Term>,
}

impl Contract {
    /// Substitutes an identifier from another.
    pub fn substitution(&mut self, old_id: usize, new_id: usize) {
        self.requires
            .iter_mut()
            .for_each(|term| term.substitution(old_id, new_id));
        self.ensures
            .iter_mut()
            .for_each(|term| term.substitution(old_id, new_id));
        self.invariant
            .iter_mut()
            .for_each(|term| term.substitution(old_id, new_id));
    }

    /// Add dependencies of a contract to the graph.
    pub fn add_dependencies(&self, node_graph: &mut DiGraphMap<usize, Label>) {
        let Contract {
            requires,
            ensures,
            invariant,
        } = self;

        requires
            .iter()
            .for_each(|term| term.add_term_dependencies(node_graph));
        ensures
            .iter()
            .for_each(|term| term.add_term_dependencies(node_graph));
        invariant
            .iter()
            .for_each(|term| term.add_term_dependencies(node_graph));
    }
}

mod term {
    use super::{Term, TermKind};

    impl Term {
        /// Substitude an identifier from another.
        pub fn substitution(&mut self, old_id: usize, new_id: usize) {
            match &mut self.kind {
                TermKind::Constant { .. } => (),
                TermKind::Identifier { ref mut id } => {
                    if *id == old_id {
                        *id = new_id
                    }
                }
                TermKind::Unary { ref mut term, .. } => {
                    term.substitution(old_id, new_id);
                }
                TermKind::Binary {
                    ref mut left,
                    ref mut right,
                    ..
                } => {
                    left.substitution(old_id, new_id);
                    right.substitution(old_id, new_id);
                }
            }
        }
    }
}
