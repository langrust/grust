use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, token, Token};

use crate::ast::{ident_colon::IdentColon, pattern::Pattern};
use crate::common::operator::{BinaryOperator, UnaryOperator};
use crate::common::{constant::Constant, r#type::Type};

use super::keyword;

/// Unop expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Unop<E> {
    /// The unary operator.
    pub op: UnaryOperator,
    /// The input expression.
    pub expression: Box<E>,
}
impl<E> Unop<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        UnaryOperator::peek(input)
    }
}
impl<E> Parse for Unop<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let expression = Box::new(input.parse()?);
        Ok(Unop { op, expression })
    }
}

/// Binop expression.
///
/// TODO: precedence
#[derive(Debug, PartialEq, Clone)]
pub struct Binop<E> {
    /// The unary operator.
    pub op: BinaryOperator,
    /// The left expression.
    pub left_expression: Box<E>,
    /// The right expression.
    pub right_expression: Box<E>,
}
impl<E> Binop<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        BinaryOperator::peek(input)
    }
    pub fn parse(left_expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let right_expression = Box::new(input.parse()?);
        Ok(Binop {
            op,
            left_expression,
            right_expression,
        })
    }
}

/// IfThenElse expression.
#[derive(Debug, PartialEq, Clone)]
pub struct IfThenElse<E> {
    /// The test expression.
    pub expression: Box<E>,
    /// The 'true' expression.
    pub true_expression: Box<E>,
    /// The 'false' expression.
    pub false_expression: Box<E>,
}
impl<E> IfThenElse<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![if])
    }
}
impl<E> Parse for IfThenElse<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![if] = input.parse()?;
        let expression = Box::new(input.parse()?);
        let _: keyword::then = input.parse()?;
        let true_expression = Box::new(input.parse()?);
        let _: Token![else] = input.parse()?;
        let false_expression = Box::new(input.parse()?);
        Ok(IfThenElse {
            expression,
            true_expression,
            false_expression,
        })
    }
}

/// Application expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Application<E> {
    /// The expression applied.
    pub function_expression: Box<E>,
    /// The inputs to the expression.
    pub inputs: Vec<E>,
}
impl<E> Application<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Paren)
    }

    pub fn parse(function_expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::parenthesized!(content in input);
        let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Application {
            function_expression,
            inputs: inputs.into_iter().collect(),
        })
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
    pub inputs: Vec<(String, Type)>,
    /// The expression abstracted.
    pub expression: Box<E>,
}
impl<E> TypedAbstraction<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![|])
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
    pub name: String,
    /// The fields associated with their expressions.
    pub fields: Vec<(String, E)>,
}
impl<E> Structure<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.call(Structure::<E>::parse).is_ok()
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
    pub elements: Vec<E>,
}
impl<E> Tuple<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Paren)
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
    pub enum_name: String,
    /// The enumeration element.
    pub elem_name: String,
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
    pub elements: Vec<E>,
}
impl<E> Array<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Bracket)
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
    pub pattern: Pattern,
    /// The optional guard.
    pub guard: Option<E>,
    /// The expression.
    pub expression: E,
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
    pub expression: Box<E>,
    /// The different matching cases.
    pub arms: Vec<Arm<E>>,
}
impl<E> Match<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![match])
    }
}
impl<E> Parse for Match<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![match] = input.parse()?;
        let expression = input.parse()?;
        let content;
        let _ = braced!(content in input);
        let arms: Punctuated<Arm<E>, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Match {
            expression: Box::new(expression),
            arms: arms.into_iter().collect(),
        })
    }
}

/// Field access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<E> {
    /// The structure expression.
    pub expression: Box<E>,
    /// The field to access.
    pub field: String,
}
impl<E> FieldAccess<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.call(syn::Ident::parse).is_ok()
    }

    pub fn parse(expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let field: syn::Ident = input.parse()?;
        Ok(FieldAccess {
            expression,
            field: field.to_string(),
        })
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
    pub expression: Box<E>,
    /// The element to access.
    pub element_number: usize,
}
impl<E> TupleElementAccess<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.call(syn::LitInt::parse).is_ok()
    }

    pub fn parse(expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let element_number: syn::LitInt = input.parse()?;
        Ok(TupleElementAccess {
            expression,
            element_number: element_number.base10_parse().unwrap(),
        })
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
    pub expression: Box<E>,
    /// The function expression.
    pub function_expression: Box<E>,
}
impl<E> Map<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::map)
    }

    pub fn parse(expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
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
    pub expression: Box<E>,
    /// The initialization expression.
    pub initialization_expression: Box<E>,
    /// The function expression.
    pub function_expression: Box<E>,
}
impl<E> Fold<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::fold)
    }

    pub fn parse(expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let _: keyword::fold = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let initialization_expression = Box::new(content.parse()?);
        let _: Token![,] = content.parse()?;
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
        let _: Token![,] = content.parse()?;
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
    pub expression: Box<E>,
    /// The function expression.
    pub function_expression: Box<E>,
}
impl<E> Sort<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::sort)
    }

    pub fn parse(expression: Box<E>, input: syn::parse::ParseStream) -> syn::Result<Self> {
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
    pub arrays: Vec<E>,
}
impl<E> Zip<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::zip)
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
    /// Unop expression.
    Unop(Unop<Expression>),
    /// Binop expression.
    Binop(Binop<Expression>),
    /// IfThenElse expression.
    IfThenElse(IfThenElse<Expression>),
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
        // TODO: add 'if then else', unop and binop !
        let mut expression = if TypedAbstraction::<Expression>::peek(input) {
            Expression::TypedAbstraction(input.parse()?)
        } else if Unop::<Expression>::peek(input) {
            Expression::Unop(input.parse()?)
        } else if IfThenElse::<Expression>::peek(input) {
            Expression::IfThenElse(input.parse()?)
        } else if Zip::<Expression>::peek(input) {
            Expression::Zip(input.parse()?)
        } else if Match::<Expression>::peek(input) {
            Expression::Match(input.parse()?)
        } else if Tuple::<Expression>::peek(input) {
            Expression::Tuple(input.parse()?)
        } else if Array::<Expression>::peek(input) {
            Expression::Array(input.parse()?)
        } else if Structure::<Expression>::peek(input) {
            Expression::Structure(input.parse()?)
        } else if Enumeration::peek(input) {
            Expression::Enumeration(input.parse()?)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Expression::Identifier(ident.to_string())
        } else if input.fork().call(Constant::parse).is_ok() {
            Expression::Constant(input.parse()?)
        } else {
            return Err(input.error("expected expression"));
        };

        loop {
            if Binop::<Expression>::peek(input) {
                expression =
                    Expression::Binop(Binop::<Expression>::parse(Box::new(expression), input)?);
            } else if Sort::<Expression>::peek(input) {
                expression =
                    Expression::Sort(Sort::<Expression>::parse(Box::new(expression), input)?);
            } else if Map::<Expression>::peek(input) {
                expression = Expression::Map(Map::<Expression>::parse(Box::new(expression), input)?)
            } else if Fold::<Expression>::peek(input) {
                expression =
                    Expression::Fold(Fold::<Expression>::parse(Box::new(expression), input)?)
            } else if TupleElementAccess::<Expression>::peek(input) {
                expression = Expression::TupleElementAccess(
                    TupleElementAccess::<Expression>::parse(Box::new(expression), input)?,
                )
            } else if FieldAccess::<Expression>::peek(input) {
                expression = Expression::FieldAccess(FieldAccess::<Expression>::parse(
                    Box::new(expression),
                    input,
                )?)
            } else if Application::<Expression>::peek(input) {
                expression = Expression::Application(Application::<Expression>::parse(
                    Box::new(expression),
                    input,
                )?)
            } else {
                break;
            }
        }
        Ok(expression)
    }
}

#[cfg(test)]
mod parse_expression {
    use crate::{
        ast::{
            expression::{
                Application, Arm, Array, Enumeration, Expression, FieldAccess, Fold, Map, Match,
                Sort, Structure, Tuple, TupleElementAccess, TypedAbstraction, Zip,
            },
            pattern::{self, Pattern},
        },
        common::{constant::Constant, r#type::Type},
    };

    #[test]
    fn should_parse_constant() {
        let expression: Expression = syn::parse_quote! {1};
        let control = Expression::Constant(Constant::Integer(syn::parse_quote! {1}));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_identifier() {
        let expression: Expression = syn::parse_quote! {x};
        let control = Expression::Identifier(String::from("x"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_application() {
        let expression: Expression = syn::parse_quote! {f(x)};
        let control = Expression::Application(Application {
            function_expression: Box::new(Expression::Identifier(String::from("f"))),
            inputs: vec![Expression::Identifier(String::from("x"))],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_typed_abstraction() {
        let expression: Expression = syn::parse_quote! {|x: int| f(x)};
        let control = Expression::TypedAbstraction(TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Application(Application {
                function_expression: Box::new(Expression::Identifier(String::from("f"))),
                inputs: vec![Expression::Identifier(String::from("x"))],
            })),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_structure() {
        let expression: Expression = syn::parse_quote! {Point {x: 0, y: 1}};
        let control = Expression::Structure(Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Constant(Constant::Integer(syn::parse_quote! {0})),
                ),
                (
                    String::from("y"),
                    Expression::Constant(Constant::Integer(syn::parse_quote! {1})),
                ),
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple() {
        let expression: Expression = syn::parse_quote! {(x, 0)};
        let control = Expression::Tuple(Tuple {
            elements: vec![
                Expression::Identifier(String::from("x")),
                Expression::Constant(Constant::Integer(syn::parse_quote! {0})),
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_enumeration() {
        let expression: Expression = syn::parse_quote! {Color::Pink};
        let control = Expression::Enumeration(Enumeration {
            enum_name: String::from("Color"),
            elem_name: String::from("Pink"),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_array() {
        let expression: Expression = syn::parse_quote! {[1, 2, 3]};
        let control = Expression::Array(Array {
            elements: vec![
                Expression::Constant(Constant::Integer(syn::parse_quote! {1})),
                Expression::Constant(Constant::Integer(syn::parse_quote! {2})),
                Expression::Constant(Constant::Integer(syn::parse_quote! {3})),
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_match() {
        let expression: Expression = syn::parse_quote! {
            match a {
                Point {x: 0, y: _} => 0,
                Point {x: x, y: _} if f(x) => -1,
                _ => 1,
            }
        };
        let control = Expression::Match(Match {
            expression: Box::new(Expression::Identifier(String::from("a"))),
            arms: vec![
                Arm {
                    pattern: Pattern::Structure(pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant(Constant::Integer(syn::parse_quote! {0})),
                            ),
                            (String::from("y"), Pattern::Default),
                        ],
                    }),
                    guard: None,
                    expression: Expression::Constant(Constant::Integer(syn::parse_quote! {0})),
                },
                Arm {
                    pattern: Pattern::Structure(pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (String::from("x"), Pattern::Identifier(String::from("x"))),
                            (String::from("y"), Pattern::Default),
                        ],
                    }),
                    guard: Some(Expression::Application(Application {
                        function_expression: Box::new(Expression::Identifier(String::from("f"))),
                        inputs: vec![Expression::Identifier(String::from("x"))],
                    })),
                    expression: Expression::Constant(Constant::Integer(syn::parse_quote! {-1})),
                },
                Arm {
                    pattern: Pattern::Default,
                    guard: None,
                    expression: Expression::Constant(Constant::Integer(syn::parse_quote! {1})),
                },
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_field_access() {
        let expression: Expression = syn::parse_quote! {p.x};
        let control = Expression::FieldAccess(FieldAccess {
            expression: Box::new(Expression::Identifier(String::from("p"))),
            field: String::from("x"),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple_element_access() {
        let expression: Expression = syn::parse_quote! {t.0};
        let control = Expression::TupleElementAccess(TupleElementAccess {
            expression: Box::new(Expression::Identifier(String::from("t"))),
            element_number: 0,
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_map() {
        let expression: Expression = syn::parse_quote! {a.map(f)};
        let control = Expression::Map(Map {
            expression: Box::new(Expression::Identifier(String::from("a"))),
            function_expression: Box::new(Expression::Identifier(String::from("f"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_fold() {
        let expression: Expression = syn::parse_quote! {a.fold(0, sum)};
        let control = Expression::Fold(Fold {
            expression: Box::new(Expression::Identifier(String::from("a"))),
            initialization_expression: Box::new(Expression::Constant(Constant::Integer(
                syn::parse_quote! {0},
            ))),
            function_expression: Box::new(Expression::Identifier(String::from("sum"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_sort() {
        let expression: Expression = syn::parse_quote! {a.sort(order)};
        let control = Expression::Sort(Sort {
            expression: Box::new(Expression::Identifier(String::from("a"))),
            function_expression: Box::new(Expression::Identifier(String::from("order"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_zip() {
        let expression: Expression = syn::parse_quote! {zip(a, b, c)};
        let control = Expression::Zip(Zip {
            arrays: vec![
                Expression::Identifier(String::from("a")),
                Expression::Identifier(String::from("b")),
                Expression::Identifier(String::from("c")),
            ],
        });
        assert_eq!(expression, control)
    }
}
