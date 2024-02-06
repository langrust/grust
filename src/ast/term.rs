use crate::common::{constant::Constant, location::Location, operator::BinaryOperator};

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
