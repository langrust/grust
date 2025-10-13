use crate::common::{
    constant::Constant,
    location::Location,
    operator::{BinaryOperator, UnaryOperator},
};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
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

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Contract's term.
pub struct Term {
    /// The kind of the term
    pub kind: TermKind,
    /// The location in source code
    pub location: Location,
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize)]
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
    /// Substitude an identifier from another.
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
