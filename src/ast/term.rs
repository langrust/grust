use crate::common::{constant::Constant, location::Location, operator::BinaryOperator};

pub(crate) enum ClauseKind { Assert, Invariant, Requires, Ensures }

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum TermKind {
    Binary {
        op: BinaryOperator,
        left: Box<Term>,
        right: Box<Term>,
    },
    Constant {
        constant: Constant,
    },
    Variable {
        id: String,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Term {
    pub kind: TermKind,
    pub location: Location,
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Contract {
    pub requires: Vec<Term>,
    pub ensures: Vec<Term>,
    pub invariant: Vec<Term>,
    pub assert: Vec<Term>,
}