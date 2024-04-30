use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, token, Token};

use crate::ast::{ident_colon::IdentColon, pattern::Pattern};
use crate::common::{constant::Constant, r#type::Type};

use super::keyword;

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

/// Arm for matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Arm<E> {
    /// The pattern to match.
    pattern: Pattern,
    /// The optional guard.
    guard: Option<E>,
    /// The expression.
    expression: E,
}
impl<E> Parse for Arm<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let _: Token![if] = input.parse()?;
                let guard = input.parse()?;
                Some(guard)
            } else {
                None
            }
        };
        let _: Token![=>] = input.parse()?;
        let expression = input.parse()?;
        Ok(Arm {
            pattern,
            guard,
            expression,
        })
    }
}

/// Pattern matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Match<E> {
    /// The expression to match.
    expression: Box<E>,
    /// The different matching cases.
    arms: Vec<Arm<E>>,
}
impl<E> Match<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(Token![match])
    }
}
impl<E> Parse for Match<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![match] = input.parse()?;
        let expression = Box::new(input.parse()?);
        let content;
        let _ = braced!(content in input);
        let arms: Punctuated<Arm<E>, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Match {
            expression,
            arms: arms.into_iter().collect(),
        })
    }
}

/// Field access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<E> {
    /// The structure expression.
    expression: Box<E>,
    /// The field to access.
    field: String,
}
impl<E> FieldAccess<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(E::parse).is_err() {
            return false;
        }
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.call(syn::Ident::parse).is_ok()
    }
}
impl<E> Parse for FieldAccess<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = Box::new(input.parse()?);
        let _: Token![.] = input.parse()?;
        let field: syn::Ident = input.parse()?;
        Ok(FieldAccess {
            expression,
            field: field.to_string(),
        })
    }
}

/// Tuple element access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleElementAccess<E> {
    /// The tuple expression.
    expression: Box<E>,
    /// The element to access.
    element_number: usize,
}
impl<E> TupleElementAccess<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(E::parse).is_err() {
            return false;
        }
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.call(syn::LitInt::parse).is_ok()
    }
}
impl<E> Parse for TupleElementAccess<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = Box::new(input.parse()?);
        let _: Token![.] = input.parse()?;
        let element_number: syn::LitInt = input.parse()?;
        Ok(TupleElementAccess {
            expression,
            element_number: element_number.base10_parse().unwrap(),
        })
    }
}

/// Array map operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Map<E> {
    /// The array expression.
    expression: Box<E>,
    /// The function expression.
    function_expression: Box<E>,
}
impl<E> Map<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(E::parse).is_err() {
            return false;
        }
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::map)
    }
}
impl<E> Parse for Map<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = Box::new(input.parse()?);
        let _: Token![.] = input.parse()?;
        let _: keyword::map = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let function_expression = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(Map {
                expression,
                function_expression,
            })
        } else {
            Err(input.error("expected only one expression"))
        }
    }
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
impl<E> Fold<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(E::parse).is_err() {
            return false;
        }
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::fold)
    }
}
impl<E> Parse for Fold<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = Box::new(input.parse()?);
        let _: Token![.] = input.parse()?;
        let _: keyword::fold = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let initialization_expression = Box::new(content.parse()?);
        let _: Token![,] = input.parse()?;
        let function_expression = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(Fold {
                expression,
                initialization_expression,
                function_expression,
            })
        } else {
            Err(input.error("expected only two expressions"))
        }
    }
}

/// Array sort operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Sort<E> {
    /// The array expression.
    expression: Box<E>,
    /// The function expression.
    function_expression: Box<E>,
}
impl<E> Sort<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(E::parse).is_err() {
            return false;
        }
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::sort)
    }
}
impl<E> Parse for Sort<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = Box::new(input.parse()?);
        let _: Token![.] = input.parse()?;
        let _: keyword::sort = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let function_expression = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(Sort {
                expression,
                function_expression,
            })
        } else {
            Err(input.error("expected only one expression"))
        }
    }
}

/// Arrays zip operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Zip<E> {
    /// The array expressions.
    arrays: Vec<E>,
}
impl<E> Zip<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(keyword::zip)
    }
}
impl<E> Parse for Zip<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: keyword::zip = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let arrays: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Zip {
            arrays: arrays.into_iter().collect(),
        })
    }
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
        if TypedAbstraction::<Expression>::peek(input) {
            Ok(Expression::TypedAbstraction(input.parse()?))
        } else if Sort::<Expression>::peek(input) {
            Ok(Expression::Sort(input.parse()?))
        } else if Map::<Expression>::peek(input) {
            Ok(Expression::Map(input.parse()?))
        } else if Fold::<Expression>::peek(input) {
            Ok(Expression::Fold(input.parse()?))
        } else if TupleElementAccess::<Expression>::peek(input) {
            Ok(Expression::TupleElementAccess(input.parse()?))
        } else if FieldAccess::<Expression>::peek(input) {
            Ok(Expression::FieldAccess(input.parse()?))
        } else if Zip::<Expression>::peek(input) {
            Ok(Expression::Zip(input.parse()?))
        } else if Application::<Expression>::peek(input) {
            Ok(Expression::Application(input.parse()?))
        } else if Match::<Expression>::peek(input) {
            Ok(Expression::Match(input.parse()?))
        } else if Tuple::<Expression>::peek(input) {
            Ok(Expression::Tuple(input.parse()?))
        } else if Array::<Expression>::peek(input) {
            Ok(Expression::Array(input.parse()?))
        } else if Structure::<Expression>::peek(input) {
            Ok(Expression::Structure(input.parse()?))
        } else if Enumeration::peek(input) {
            Ok(Expression::Enumeration(input.parse()?))
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Ok(Expression::Identifier(ident.to_string()))
        } else if input.fork().call(Constant::parse).is_ok() {
            Ok(Expression::Constant(input.parse()?))
        } else {
            Err(input.error("expected expression"))
        }
    }
}
