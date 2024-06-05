prelude! {
    syn::{
        parse::Parse,
        punctuated::Punctuated,
        {braced, parenthesized, token, Token},
    },
    contract::Contract,
    Stmt,
}

use super::colon::Colon;
use super::keyword;

/// GRust function AST.
pub struct Function {
    pub function_token: keyword::function,
    /// Function identifier.
    pub ident: syn::Ident,
    pub args_paren: token::Paren,
    /// Function's inputs identifiers and their types.
    pub args: Punctuated<Colon<syn::Ident, Typ>, Token![,]>,
    pub arrow_token: Token![->],
    pub output_type: Typ,
    /// Function's contract.
    pub contract: Contract,
    pub brace: token::Brace,
    /// Function's statements.
    pub statements: Vec<Stmt>,
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
        let args: Punctuated<Colon<syn::Ident, Typ>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        let arrow_token: Token![->] = input.parse()?;
        let output_type: Typ = input.parse()?;
        let contract: Contract = input.parse()?;
        let content;
        let brace: token::Brace = braced!(content in input);
        let statements: Vec<Stmt> = {
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
            contract,
            brace,
            statements,
        })
    }
}
