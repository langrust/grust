//! HIR [Contract](crate::hir::contract::Contract) module.

pub mod term {
    //! HIR [Term](crate::hir::contract::term::Term) module.
    prelude! {
        operator::{BinaryOperator, UnaryOperator},
        graph::*,
    }

    #[derive(Debug, PartialEq, Clone)]
    /// Term's kind.
    pub enum Kind {
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
        /// Enumeration term
        Enumeration {
            /// The enumeration id.
            enum_id: usize,
            /// The element id.
            element_id: usize,
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
        /// Forall term: forall x, P(x)
        ForAll { id: usize, term: Box<Term> },
        /// Implication term: P => Q
        Implication { left: Box<Term>, right: Box<Term> },
        /// Present event pattern
        PresentEvent {
            /// The event identifier
            event_id: usize,
            /// The event pattern
            pattern: usize,
        },
        /// Timeout event pattern
        TimeoutEvent {
            /// The event identifier
            event_id: usize,
        },
    }

    mk_new! { impl Kind =>
        Constant: constant { constant: Constant }
        Identifier: ident { id: usize }
        Enumeration: enumeration {
            enum_id: usize,
            element_id: usize,
        }
        Unary: unary {
            op: UnaryOperator,
            term: Term = term.into(),
        }
        Binary: binary {
            op: BinaryOperator,
            left: Term = left.into(),
            right: Term = right.into(),
        }
        ForAll: forall {
            id: usize,
            term: Term = term.into(),
        }
        Implication: implication {
            left: Term = left.into(),
            right: Term = right.into(),
        }
        PresentEvent: present {
            event_id: usize,
            pattern: usize,
        }
        TimeoutEvent: timeout {
            event_id: usize,
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    /// Contract's term.
    pub struct Term {
        /// The kind of the term
        pub kind: Kind,
        /// The type of the term
        pub typing: Option<Typ>,
        /// The location in source code
        pub location: Location,
    }

    mk_new! { impl Term =>
        new {
            kind: Kind,
            typing: Option<Typ>,
            location: Location,
        }
    }

    impl Term {
        /// Compute dependencies of a term.
        pub fn compute_dependencies(&self) -> Vec<usize> {
            match &self.kind {
                Kind::Unary { term, .. } => term.compute_dependencies(),
                Kind::Binary { left, right, .. } | Kind::Implication { left, right, .. } => {
                    let mut dependencies_left = left.compute_dependencies();
                    let mut dependencies = right.compute_dependencies();
                    dependencies.append(&mut dependencies_left);
                    dependencies
                }
                Kind::Constant { .. } | Kind::Enumeration { .. } | Kind::TimeoutEvent { .. } => {
                    vec![]
                }
                Kind::Identifier { id } | Kind::PresentEvent { pattern: id, .. } => vec![*id],
                Kind::ForAll { id, term, .. } => term
                    .compute_dependencies()
                    .into_iter()
                    .filter(|signal| id != signal)
                    .collect(),
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

    impl Term {
        /// Substitude an identifier with another one.
        pub fn substitution(&mut self, old_id: usize, new_id: usize) {
            match &mut self.kind {
                Kind::Constant { .. } | Kind::Enumeration { .. } | Kind::TimeoutEvent { .. } => (),
                Kind::Identifier { ref mut id }
                | Kind::PresentEvent {
                    pattern: ref mut id,
                    ..
                } => {
                    if *id == old_id {
                        *id = new_id
                    }
                }
                Kind::Unary { ref mut term, .. } => {
                    term.substitution(old_id, new_id);
                }
                Kind::Binary {
                    ref mut left,
                    ref mut right,
                    ..
                }
                | Kind::Implication {
                    ref mut left,
                    ref mut right,
                    ..
                } => {
                    left.substitution(old_id, new_id);
                    right.substitution(old_id, new_id);
                }
                Kind::ForAll { id, term, .. } => {
                    if old_id != *id {
                        term.substitution(old_id, new_id)
                    }
                    // if 'id to replace' is equal to 'id of the forall' then nothing to do
                }
            }
        }
    }
}

prelude! {
    just graph::*,
}

pub use term::Term;

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
