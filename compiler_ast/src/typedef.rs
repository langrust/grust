prelude! {
    syn::{Parse, Punctuated, token, Token},
}

use super::{colon::Colon, keyword};

/// GRust user defined type AST.
pub enum Typedef {
    /// Represents a structure definition.
    Structure {
        struct_token: Token![struct],
        /// Typedef identifier.
        ident: Ident,
        brace: token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        fields: Punctuated<Colon<Ident, Typ>, Token![,]>,
    },
    /// Represents an enumeration definition.
    Enumeration {
        enum_token: Token![enum],
        /// Typedef identifier.
        ident: Ident,
        brace: token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        elements: Punctuated<Ident, Token![,]>,
    },
    /// Represents an array definition.
    Array {
        array_token: keyword::array,
        /// Typedef identifier.
        ident: Ident,
        bracket_token: token::Bracket,
        /// The array's type.
        array_type: Typ,
        semi_token: Token![;],
        /// The array's size.
        size: syn::LitInt,
    },
}
impl Typedef {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![struct]) || input.peek(Token![enum]) || input.peek(keyword::array)
    }
}
impl Parse for Typedef {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![struct]) {
            let struct_token: Token![struct] = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let fields: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Typedef::Structure {
                struct_token,
                ident,
                brace,
                fields,
            })
        } else if input.peek(Token![enum]) {
            let enum_token: Token![enum] = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let elements: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Typedef::Enumeration {
                enum_token,
                ident,
                brace,
                elements,
            })
        } else if input.peek(keyword::array) {
            let array_token: keyword::array = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let bracket_token: token::Bracket = bracketed!(content in input);
            let array_type: Typ = content.parse()?;
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
