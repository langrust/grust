use syn::{braced, bracketed, parse::Parse, punctuated::Punctuated, token, Token};

use crate::common::r#type::Type;

use super::{ident_colon::IdentColon, keyword};

/// GRust user defined type AST.
pub enum Typedef {
    /// Represents a structure definition.
    Structure {
        struct_token: Token![struct],
        /// Typedef identifier.
        ident: syn::Ident,
        brace: token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        fields: Punctuated<IdentColon<Type>, Token![,]>,
    },
    /// Represents an enumeration definition.
    Enumeration {
        enum_token: Token![enum],
        /// Typedef identifier.
        ident: syn::Ident,
        brace: token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        elements: Punctuated<syn::Ident, Token![,]>,
    },
    /// Represents an array definition.
    Array {
        array_token: keyword::array,
        /// Typedef identifier.
        ident: syn::Ident,
        bracket_token: token::Bracket,
        /// The array's type.
        array_type: Type,
        semi_token: Token![;],
        /// The array's size.
        size: syn::LitInt,
    },
}
impl Parse for Typedef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![struct]) {
            let struct_token: Token![struct] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let fields: Punctuated<IdentColon<Type>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Typedef::Structure {
                struct_token,
                ident,
                brace,
                fields,
            })
        } else if input.peek(Token![enum]) {
            let enum_token: Token![enum] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let elements: Punctuated<syn::Ident, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Typedef::Enumeration {
                enum_token,
                ident,
                brace,
                elements,
            })
        } else if input.peek(keyword::array) {
            let array_token: keyword::array = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            let content;
            let bracket_token: token::Bracket = bracketed!(content in input);
            let array_type: Type = content.parse()?;
            let semi_token: Token![;] = content.parse()?;
            let size: syn::LitInt = content.parse()?;
            if content.is_empty() {
                Ok(Typedef::Array {
                    array_token,
                    ident,
                    bracket_token,
                    array_type,
                    semi_token,
                    size,
                })
            } else {
                Err(input.error("expected array alias definition"))
            }
        } else {
            Err(input.error("expected type definition"))
        }
    }
}
