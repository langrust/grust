prelude! {
    syn::{
        Result,
        parse::Parse,
        punctuated::Punctuated,
        braced, parenthesized, token, LitInt, Token,
    },
    contract::Contract, equation::Equation,
}

use super::colon::Colon;
use super::keyword;

/// GRust component AST.
pub struct Component {
    pub component_token: keyword::component,
    /// Component identifier.
    pub ident: syn::Ident,
    pub args_paren: token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: Punctuated<Colon<syn::Ident, Typ>, Token![,]>,
    pub arrow_token: Token![->],
    pub outs_paren: token::Paren,
    /// Component's outputs identifiers and their types.
    pub outs: Punctuated<Colon<syn::Ident, Typ>, Token![,]>,
    /// Component's computation period.
    pub period: Option<(Token![@], LitInt, keyword::ms)>,
    /// Component's contract.
    pub contract: Contract,
    pub brace: token::Brace,
    /// Component's equations.
    pub equations: Vec<Equation>,
}
impl Component {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::component)
    }
}
impl Parse for Component {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let component_token: keyword::component = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let content;
        let args_paren: token::Paren = parenthesized!(content in input);
        let args: Punctuated<Colon<syn::Ident, Typ>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        let arrow_token: Token![->] = input.parse()?;
        let content;
        let outs_paren: token::Paren = parenthesized!(content in input);
        let outs: Punctuated<Colon<syn::Ident, Typ>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        let period: Option<(Token![@], LitInt, keyword::ms)> = {
            if input.peek(Token![@]) {
                Some((input.parse()?, input.parse()?, input.parse()?))
            } else {
                None
            }
        };
        let contract: Contract = input.parse()?;
        let content;
        let brace: token::Brace = braced!(content in input);
        let equations: Vec<Equation> = {
            let mut equations = vec![];
            while !content.is_empty() {
                equations.push(content.parse()?)
            }
            equations
        };
        Ok(Component {
            component_token,
            ident,
            args_paren,
            args,
            arrow_token,
            outs_paren,
            outs,
            period,
            contract,
            brace,
            equations,
        })
    }
}

#[cfg(test)]
mod parse_component {
    use super::*;

    #[test]
    fn should_parse_component() {
        let _: Component = syn::parse_quote! {
            component counter(res: bool, tick: bool) -> (o: int) {
                o = if res then 0 else (0 fby o) + inc;
                let inc: int = if tick then 1 else 0;
            }
        };
    }
}

/// GRust component import AST.
pub struct ComponentImport {
    pub import_token: keyword::import,
    pub component_token: keyword::component,
    pub path: syn::Path,
    pub colon_token: Token![:],
    pub args_paren: token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: Punctuated<Colon<syn::Ident, Typ>, Token![,]>,
    pub arrow_token: Token![->],
    pub outs_paren: token::Paren,
    /// Component's outputs identifiers and their types.
    pub outs: Punctuated<Colon<syn::Ident, Typ>, Token![,]>,
    /// Component's computation period.
    pub period: Option<(Token![@], LitInt, keyword::ms)>,
    pub semi_token: Token![;],
}
impl ComponentImport {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked
            .parse::<keyword::import>()
            .and_then(|_| forked.parse::<keyword::component>())
            .is_ok()
    }
}
impl Parse for ComponentImport {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let import_token: keyword::import = input.parse()?;
        let component_token: keyword::component = input.parse()?;
        let path: syn::Path = input.parse()?;
        let colon_token: Token![:] = input.parse()?;
        let content;
        let args_paren: token::Paren = parenthesized!(content in input);
        let args: Punctuated<Colon<syn::Ident, Typ>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        let arrow_token: Token![->] = input.parse()?;
        let content;
        let outs_paren: token::Paren = parenthesized!(content in input);
        let outs: Punctuated<Colon<syn::Ident, Typ>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        let period: Option<(Token![@], LitInt, keyword::ms)> = {
            if input.peek(Token![@]) {
                Some((input.parse()?, input.parse()?, input.parse()?))
            } else {
                None
            }
        };
        let semi_token: Token![;] = input.parse()?;
        Ok(ComponentImport {
            import_token,
            component_token,
            path,
            colon_token,
            args_paren,
            args,
            arrow_token,
            outs_paren,
            outs,
            period,
            semi_token,
        })
    }
}

#[cfg(test)]
mod parse_component_import {
    use super::*;

    #[test]
    fn should_parse_component() {
        let _: ComponentImport = syn::parse_quote! {
            import component std::rising_edge : (test: bool) -> (res: bool);
        };
    }
}
