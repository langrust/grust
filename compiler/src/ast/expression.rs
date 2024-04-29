use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, token, Token};

use crate::ast::{ident_colon::IdentColon, pattern::Pattern};
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
impl<E> Parse for Application<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let function_expression: Box<E> = Box::new(input.parse()?);
        let content;
        let _ = syn::parenthesized!(content in input);
        let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Application {
            function_expression,
            inputs: inputs.into_iter().collect(),
        })
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
impl<E> TypedAbstraction<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(Token![|])
    }
}
impl<E> Parse for TypedAbstraction<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![|] = input.parse()?;
        let mut inputs: Punctuated<IdentColon<Type>, Token![,]> = Punctuated::new();
        loop {
            if input.peek(Token![|]) {
                break;
            }
            let value = input.parse()?;
            inputs.push_value(value);
            if input.peek(Token![|]) {
                break;
            }
            let punct: Token![,] = input.parse()?;
            inputs.push_punct(punct);
        }
        let _: Token![|] = input.parse()?;
        let expression: E = input.parse()?;
        Ok(TypedAbstraction {
            inputs: inputs
                .into_iter()
                .map(|IdentColon { ident, elem, .. }| (ident.to_string(), elem))
                .collect(),
            expression: Box::new(expression),
        })
    }
}

/// Structure expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure<E> {
    /// The structure name.
    name: String,
    /// The fields associated with their expressions.
    fields: Vec<(String, E)>,
}
impl<E> Structure<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(token::Brace)
    }
}
impl<E> Parse for Structure<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = braced!(content in input);
        let fields: Punctuated<IdentColon<E>, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Structure {
            name: ident.to_string(),
            fields: fields
                .into_iter()
                .map(|IdentColon { ident, elem, .. }| (ident.to_string(), elem))
                .collect(),
        })
    }
}

/// Tuple expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple<E> {
    /// The elements.
    elements: Vec<E>,
}
impl<E> Tuple<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(token::Paren)
    }
}
impl<E> Parse for Tuple<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Tuple {
            elements: elements.into_iter().collect(),
        })
    }
}

/// Enumeration expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration {
    /// The enumeration name.
    enum_name: String,
    /// The enumeration element.
    elem_name: String,
}
impl Enumeration {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(Token![::])
    }
}
impl Parse for Enumeration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_enum: syn::Ident = input.parse()?;
        let _: Token![::] = input.parse()?;
        let ident_elem: syn::Ident = input.parse()?;
        Ok(Enumeration {
            enum_name: ident_enum.to_string(),
            elem_name: ident_elem.to_string(),
        })
    }
}

/// Array expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Array<E> {
    /// The elements inside the array.
    elements: Vec<E>,
}
impl<E> Array<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(token::Bracket)
    }
}
impl<E> Parse for Array<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = bracketed!(content in input);
        let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Array {
            elements: elements.into_iter().collect(),
        })
    }
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
            Ok(Expression::Application(input.parse()?))
        } else if TypedAbstraction::<Expression>::peek(input) {
            Ok(Expression::TypedAbstraction(input.parse()?))
        } else if Structure::<Expression>::peek(input) {
            Ok(Expression::Structure(input.parse()?))
        } else if Tuple::<Expression>::peek(input) {
            Ok(Expression::Tuple(input.parse()?))
        } else if Enumeration::peek(input) {
            Ok(Expression::Enumeration(input.parse()?))
        } else if Array::<Expression>::peek(input) {
            Ok(Expression::Array(input.parse()?))
        } else {
            todo!()
        }
    }
}
