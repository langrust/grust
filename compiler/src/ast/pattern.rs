use syn::{braced, parenthesized, parse::Parse, punctuated::Punctuated, token, Token};

use crate::common::constant::Constant;

use super::{ident_colon::IdentColon, keyword};

/// Structure pattern that matches the structure and its fields.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure {
    /// The structure name.
    pub name: String,
    /// The structure fields with the corresponding patterns to match.
    pub fields: Vec<(String, Pattern)>,
}
impl Structure {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(token::Brace)
    }
}
impl Parse for Structure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = braced!(content in input);
        let fields: Punctuated<IdentColon<Pattern>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        Ok(Structure {
            name: ident.to_string(),
            fields: fields
                .into_iter()
                .map(|IdentColon { ident, elem, .. }| (ident.to_string(), elem))
                .collect(),
        })
    }
}

/// Enumeration pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration {
    /// The enumeration type name.
    pub enum_name: String,
    /// The element name.
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

/// Tuple pattern that matches tuples.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
impl Tuple {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(token::Paren)
    }
}
impl Parse for Tuple {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Tuple {
            elements: elements.into_iter().collect(),
        })
    }
}

/// Some pattern that matches when an optional has a value which match the pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct Some {
    /// The pattern matching the value.
    pub pattern: Box<Pattern>,
}
impl Some {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.peek(keyword::some)
    }
}
impl Parse for Some {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: keyword::some = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let pattern = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(Some { pattern })
        } else {
            Err(input.error("expected only one pattern"))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub enum Pattern {
    /// Constant pattern.
    Constant(Constant),
    /// Identifier pattern.
    Identifier(String),
    /// Structure pattern that matches the structure and its fields.
    Structure(Structure),
    /// Enumeration pattern.
    Enumeration(Enumeration),
    /// Tuple pattern that matches tuples.
    Tuple(Tuple),
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some(Some),
    /// None pattern.
    None,
    /// Default pattern.
    Default,
}
impl Parse for Pattern {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.fork().call(Constant::parse).is_ok() {
            Ok(Pattern::Constant(input.parse()?))
        } else if Structure::peek(input) {
            Ok(Pattern::Structure(input.parse()?))
        } else if Tuple::peek(input) {
            Ok(Pattern::Tuple(input.parse()?))
        } else if Enumeration::peek(input) {
            Ok(Pattern::Enumeration(input.parse()?))
        } else if Some::peek(input) {
            Ok(Pattern::Some(input.parse()?))
        } else if input.fork().peek(keyword::none) {
            let _: keyword::none = input.parse()?;
            Ok(Pattern::None)
        } else if input.fork().peek(Token![_]) {
            let _: Token![_] = input.parse()?;
            Ok(Pattern::Default)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Ok(Pattern::Identifier(ident.to_string()))
        } else {
            Err(input.error("expected pattern"))
        }
    }
}
