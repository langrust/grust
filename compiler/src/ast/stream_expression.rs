use crate::ast::expression::{
    Application, Array, Enumeration, FieldAccess, Fold, Map, Match, Sort, Structure, Tuple,
    TupleElementAccess, TypedAbstraction, Zip,
};
use crate::common::constant::Constant;

/// Initialized buffer stream expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FollowedBy {
    /// The initialization constant.
    constant: Box<StreamExpression>,
    /// The buffered expression.
    expression: Box<StreamExpression>,
}

#[derive(Debug, PartialEq, Clone)]
/// GRust stream expression kind AST.
pub enum StreamExpression {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(syn::Ident),
    /// Application expression.
    Application(Application<StreamExpression>),
    /// Abstraction expression with inputs types.
    TypedAbstraction(TypedAbstraction<StreamExpression>),
    /// Structure expression.
    Structure(Structure<StreamExpression>),
    /// Tuple expression.
    Tuple(Tuple<StreamExpression>),
    /// Enumeration expression.
    Enumeration(Enumeration),
    /// Array expression.
    Array(Array<StreamExpression>),
    /// Pattern matching expression.
    Match(Match<StreamExpression>),
    /// Field access expression.
    FieldAccess(FieldAccess<StreamExpression>),
    /// Tuple element access expression.
    TupleElementAccess(TupleElementAccess<StreamExpression>),
    /// Array map operator expression.
    Map(Map<StreamExpression>),
    /// Array fold operator expression.
    Fold(Fold<StreamExpression>),
    /// Array sort operator expression.
    Sort(Sort<StreamExpression>),
    /// Arrays zip operator expression.
    Zip(Zip<StreamExpression>),
    /// Initialized buffer stream expression.
    FollowedBy(FollowedBy),
}
