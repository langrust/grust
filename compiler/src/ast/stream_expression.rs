use syn::parse::Parse;

use crate::ast::{
    expression::{
        Application, Array, Enumeration, FieldAccess, Fold, Map, Match, Sort, Structure, Tuple,
        TupleElementAccess, TypedAbstraction, Zip,
    },
    keyword,
};
use crate::common::constant::Constant;

/// Initialized buffer stream expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FollowedBy {
    /// The initialization constant.
    pub constant: Box<StreamExpression>,
    /// The buffered expression.
    pub expression: Box<StreamExpression>,
}
impl FollowedBy {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(StreamExpression::parse).is_err() {
            return false;
        }
        forked.peek(keyword::fby)
    }
}
impl Parse for FollowedBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let constant = Box::new(input.parse()?);
        let _: keyword::fby = input.parse()?;
        let expression = Box::new(input.parse()?);
        Ok(FollowedBy {
            constant,
            expression,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust stream expression kind AST.
pub enum StreamExpression {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(String),
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
impl Parse for StreamExpression {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if FollowedBy::peek(input) {
            Ok(StreamExpression::FollowedBy(input.parse()?))
        } else if TypedAbstraction::<StreamExpression>::peek(input) {
            Ok(StreamExpression::TypedAbstraction(input.parse()?))
        } else if Sort::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Sort(input.parse()?))
        } else if Map::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Map(input.parse()?))
        } else if Fold::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Fold(input.parse()?))
        } else if TupleElementAccess::<StreamExpression>::peek(input) {
            Ok(StreamExpression::TupleElementAccess(input.parse()?))
        } else if FieldAccess::<StreamExpression>::peek(input) {
            Ok(StreamExpression::FieldAccess(input.parse()?))
        } else if Zip::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Zip(input.parse()?))
        } else if Application::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Application(input.parse()?))
        } else if Match::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Match(input.parse()?))
        } else if Tuple::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Tuple(input.parse()?))
        } else if Array::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Array(input.parse()?))
        } else if Structure::<StreamExpression>::peek(input) {
            Ok(StreamExpression::Structure(input.parse()?))
        } else if Enumeration::peek(input) {
            Ok(StreamExpression::Enumeration(input.parse()?))
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Ok(StreamExpression::Identifier(ident.to_string()))
        } else if input.fork().call(Constant::parse).is_ok() {
            Ok(StreamExpression::Constant(input.parse()?))
        } else {
            Err(input.error("expected expression"))
        }
    }
}
