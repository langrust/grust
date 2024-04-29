use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{token, Token};

use crate::ast::pattern::Pattern;
use crate::common::{constant::Constant, r#type::Type};

/// Application expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Application<E> {
    /// The expression applied.
    function_expression: Box<E>,
    /// The inputs to the expression.
    inputs: Vec<E>,
}
impl<E> Application<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(E::parse).is_err() {
            return false;
        }
        forked.peek(token::Paren)
    }
}

/// Abstraction expression with inputs types.
#[derive(Debug, PartialEq, Clone)]
pub struct TypedAbstraction<E> {
    /// The inputs to the abstraction.
    inputs: Vec<(String, Type)>,
    /// The expression abstracted.
    expression: Box<E>,
}
/// Structure expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure<E> {
    /// The structure name.
    name: String,
    /// The fields associated with their expressions.
    fields: Vec<(String, E)>,
}
/// Tuple expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple<E> {
    /// The elements.
    elements: Vec<E>,
}
/// Enumeration expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration {
    /// The enumeration name.
    enum_name: String,
    /// The enumeration element.
    elem_name: String,
}
/// Array expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Array<E> {
    /// The elements inside the array.
    elements: Vec<E>,
}
/// Pattern matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Match<E> {
    /// The expression to match.
    expression: Box<E>,
    /// The different matching cases.
    arms: Vec<(Pattern, Option<E>, E)>,
}
/// When present expression.
#[derive(Debug, PartialEq, Clone)]
pub struct When<E> {
    /// The identifier of the value when present
    id: String,
    /// The optional expression.
    option: Box<E>,
    /// The expression when present.
    present: Box<E>,
    /// The default expression.
    default: Box<E>,
}
/// Field access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<E> {
    /// The structure expression.
    expression: Box<E>,
    /// The field to access.
    field: String,
}
/// Tuple element access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleElementAccess<E> {
    /// The tuple expression.
    expression: Box<E>,
    /// The element to access.
    element_number: usize,
}
/// Array map operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Map<E> {
    /// The array expression.
    expression: Box<E>,
    /// The function expression.
    function_expression: Box<E>,
}
/// Array fold operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Fold<E> {
    /// The array expression.
    expression: Box<E>,
    /// The initialization expression.
    initialization_expression: Box<E>,
    /// The function expression.
    function_expression: Box<E>,
}
/// Array sort operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Sort<E> {
    /// The array expression.
    expression: Box<E>,
    /// The function expression.
    function_expression: Box<E>,
}
/// Arrays zip operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Zip<E> {
    /// The array expressions.
    arrays: Vec<E>,
}

#[derive(Debug, PartialEq, Clone)]
/// GRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(String),
    /// Application expression.
    Application(Application<Expression>),
    /// Abstraction expression with inputs types.
    TypedAbstraction(TypedAbstraction<Expression>),
    /// Structure expression.
    Structure(Structure<Expression>),
    /// Tuple expression.
    Tuple(Tuple<Expression>),
    /// Enumeration expression.
    Enumeration(Enumeration),
    /// Array expression.
    Array(Array<Expression>),
    /// Pattern matching expression.
    Match(Match<Expression>),
    /// When present expression.
    When(When<Expression>),
    /// Field access expression.
    FieldAccess(FieldAccess<Expression>),
    /// Tuple element access expression.
    TupleElementAccess(TupleElementAccess<Expression>),
    /// Array map operator expression.
    Map(Map<Expression>),
    /// Array fold operator expression.
    Fold(Fold<Expression>),
    /// Array sort operator expression.
    Sort(Sort<Expression>),
    /// Arrays zip operator expression.
    Zip(Zip<Expression>),
}

impl Parse for Expression {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.fork().call(Constant::parse).is_ok() {
            Ok(Expression::Constant(input.parse()?))
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Ok(Expression::Identifier(ident.to_string()))
        } else if Application::<Expression>::peek(input) {
            let function_expression: Box<Expression> = Box::new(input.parse()?);
            let content;
            let _ = syn::parenthesized!(content in input);
            let inputs: Punctuated<Expression, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Expression::Application(Application {
                function_expression,
                inputs: inputs.into_iter().collect(),
            }))
        } else {
            todo!()
        }
    }
}
