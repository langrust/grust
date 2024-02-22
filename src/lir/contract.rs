use crate::common::{
    constant::Constant,
    operator::{BinaryOperator, UnaryOperator},
    scope::Scope,
};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Term.
pub enum Term {
    /// Constant term: 3
    Constant {
        /// The constant
        constant: Constant,
    },
    /// Identifier term: x
    Identifier {
        /// The identifier's name.
        name: String,
        /// The identifier's scope.
        scope: Scope,
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
