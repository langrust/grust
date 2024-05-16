use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Token};

use crate::ast::statement::Statement;
use crate::common::r#type::Type;

use super::colon::Colon;
use super::keyword;

/// GRust function AST.
pub struct Function {
    pub function_token: keyword::function,
    /// Function identifier.
    pub ident: syn::Ident,
    pub args_paren: token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: Punctuated<Colon<syn::Ident, Type>, Token![,]>,
    pub arrow_token: Token![->],
    pub output_type: Type,
    pub brace: token::Brace,
    /// Function's statements.
    pub statements: Vec<Statement>,
}
impl Function {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::function)
    }
}
impl Parse for Function {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let function_token: keyword::function = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let content;
        let args_paren: token::Paren = parenthesized!(content in input);
        let args: Punctuated<Colon<syn::Ident, Type>, Token![,]> = Punctuated::parse_terminated(&content)?;
        let arrow_token: Token![->] = input.parse()?;
        let output_type: Type = input.parse()?;
        let content;
        let brace: token::Brace = braced!(content in input);
        let statements: Vec<Statement> = {
            let mut statements = Vec::new();
            while !content.is_empty() {
                statements.push(content.parse()?);
            }
            statements
        };
        Ok(Function {
            function_token,
            ident,
            args_paren,
            args,
            arrow_token,
            output_type,
            brace,
            statements,
        })
    }
}
