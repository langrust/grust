use crate::common::{constant::Constant, location::Location, operator::BinaryOperator};

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
    pub fn substitution(&mut self, id: usize, new_id: usize) {
        todo!()
    }
}
