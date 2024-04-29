use syn::{punctuated::Punctuated, token, Token};

use crate::common::r#type::Type;

use super::keyword;

#[derive(Debug, PartialEq, Clone)]
/// GRust user defined type AST.
pub enum Typedef {
    /// Represents a structure definition.
    Structure {
        struct_token: Token![struct],
        /// Typedef identifier.
        ident: syn::Ident,
        brace: token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        fields: Punctuated<(syn::Ident, Token![:], Type), Token![,]>,
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
