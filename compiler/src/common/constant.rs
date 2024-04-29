use syn::parse::Parse;

use crate::common::r#type::Type;

/// GRust constants.
///
/// [Constant] enumeration is used to describe GRust expressions.
///
/// It reprensents all possible constant:
/// - [Constant::Integer] are [i64] integers, `1` becomes `Constant::Integer(1)`
/// - [Constant::Float] are [f64] floats, `1.0` becomes `Constant::Float(1.0)`
/// - [Constant::Boolean] is the [bool] type for booleans, `true` becomes `Constant::Boolean(true)`
/// - [Constant::Unit] is the unit type, `()` becomes `Constant::Unit`
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// [i64] integers
    Integer(syn::LitInt),
    /// [f64] floats
    Float(syn::LitFloat),
    /// [bool] booleans
    Boolean(syn::LitBool),
    /// Unit constant
    Unit(syn::token::Paren),
}
impl Constant {
    /// Get the [Type] of the constant.
    pub fn get_type(&self) -> Type {
        match self {
            Constant::Integer(_) => Type::Integer,
            Constant::Float(_) => Type::Float,
            Constant::Boolean(_) => Type::Boolean,
            Constant::Unit(_) => Type::Unit,
        }
    }
}
impl Parse for Constant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.fork().call(syn::LitInt::parse).is_ok() {
            let i: syn::LitInt = input.parse()?;
            Ok(Constant::Integer(i))
        } else if input.fork().call(syn::LitFloat::parse).is_ok() {
            let f: syn::LitFloat = input.parse()?;
            Ok(Constant::Float(f))
        } else if input.fork().call(syn::LitBool::parse).is_ok() {
            let b: syn::LitBool = input.parse()?;
            Ok(Constant::Boolean(b))
        } else {
            let content;
            let parens = syn::parenthesized!(content in input);
            if content.is_empty() {
                if input.is_empty() {
                    Ok(Constant::Unit(parens))
                } else {
                    Err(input.error("expected constant: integer, float, boolean or unit"))
                }
            } else {
                Err(input.error("expected unit `()`"))
            }
        }
    }
}
