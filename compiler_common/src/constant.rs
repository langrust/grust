prelude! {
    syn::{
        LitInt, LitFloat, LitBool, token::Paren, parse::Parse, spanned::Spanned
    },
}

/// GRust constants.
///
/// [Constant] enumeration is used to describe GRust expressions.
///
/// It represents all possible constant:
///
/// - [Constant::Integer] are [i64] integers, `1` becomes `Constant::Integer(1)`
/// - [Constant::Float] are [f64] floats, `1.0` becomes `Constant::Float(1.0)`
/// - [Constant::Boolean] is the [bool] type for booleans, `true` becomes `Constant::Boolean(true)`
/// - [Constant::Unit] is the unit type, `()` becomes `Constant::Unit`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    /// [i64] integers
    Integer(LitInt),
    /// [f64] floats
    Float(LitFloat),
    /// [bool] booleans
    Boolean(LitBool),
    /// Unit constant
    Unit(Paren),
}
mk_new! { impl Constant =>
    Integer: int(l: LitInt = l)
    Float: float(l: LitFloat = l)
    Boolean: bool(l: LitBool = l)
    Unit: unit(l: Paren = l)
    Unit: unit_default(l = Default::default())
}

impl Constant {
    /// Get the [Typ] of the constant.
    pub fn get_type(&self) -> Typ {
        match self {
            Constant::Integer(lit) => Typ::Integer(keyword::int { span: lit.span() }),
            Constant::Float(lit) => Typ::Float(keyword::float { span: lit.span() }),
            Constant::Boolean(lit) => Typ::Boolean(keyword::bool { span: lit.span() }),
            Constant::Unit(paren) => Typ::Unit(keyword::unit {
                span: paren.span.span(),
            }),
        }
    }
}
impl Parse for Constant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            let i: syn::LitInt = input.parse()?;
            Ok(Constant::Integer(i))
        } else if input.peek(syn::LitFloat) {
            let f: syn::LitFloat = input.parse()?;
            Ok(Constant::Float(f))
        } else if input.peek(syn::LitBool) {
            let b: syn::LitBool = input.parse()?;
            Ok(Constant::Boolean(b))
        } else {
            let content;
            let parens = syn::parenthesized!(content in input);
            if content.is_empty() {
                Ok(Constant::Unit(parens))
            } else {
                Err(input.error("expected unit `()`"))
            }
        }
    }
}
