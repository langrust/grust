use syn::{parenthesized, parse::Parse, punctuated::Punctuated, token, Token};

use super::keyword;
use crate::common::r#type::Type;

/// GReact `sample` operator.
#[derive(Debug, PartialEq, Clone)]
pub struct Sample {
    sample_token: keyword::sample,
    paren_token: token::Paren,
    /// Input expression.
    flow_expression: Box<FlowExpression>,
    comma_token: Token![,],
    /// Sampling period in milliseconds.
    period_ms: syn::LitInt,
}
impl Sample {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::sample)
    }
}
impl Parse for Sample {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sample_token: keyword::sample = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let period_ms: syn::LitInt = content.parse()?;
        if content.is_empty() {
            Ok(Sample {
                sample_token,
                paren_token,
                flow_expression,
                comma_token,
                period_ms,
            })
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}

/// GReact `map` operator.
#[derive(Debug, PartialEq, Clone)]
pub struct Map {
    map_token: keyword::map,
    paren_token: token::Paren,
    /// Input expression.
    flow_expression: Box<FlowExpression>,
    comma_token: Token![,],
    /// Function to apply on each element of the flow.
    function: syn::Expr,
}
impl Map {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::map)
    }
}
impl Parse for Map {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let map_token: keyword::map = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let function: syn::Expr = content.parse()?;
        if content.is_empty() {
            Ok(Map {
                map_token,
                paren_token,
                flow_expression,
                comma_token,
                function,
            })
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}

/// GReact `merge` operator.
#[derive(Debug, PartialEq, Clone)]
pub struct Merge {
    merge_token: keyword::merge,
    paren_token: token::Paren,
    /// Input expression 1.
    flow_expression_1: Box<FlowExpression>,
    comma_token: Token![,],
    /// Input expression 2.
    flow_expression_2: Box<FlowExpression>,
}
impl Merge {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::merge)
    }
}
impl Parse for Merge {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let merge_token: keyword::merge = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression_1: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let flow_expression_2: Box<FlowExpression> = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(Merge {
                merge_token,
                paren_token,
                flow_expression_1,
                comma_token,
                flow_expression_2,
            })
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}

/// GReact `zip` operator.
#[derive(Debug, PartialEq, Clone)]
pub struct Zip {
    zip_token: keyword::zip,
    paren_token: token::Paren,
    /// Input expression 1.
    flow_expression_1: Box<FlowExpression>,
    comma_token: Token![,],
    /// Input expression 2.
    flow_expression_2: Box<FlowExpression>,
}
impl Zip {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::zip)
    }
}
impl Parse for Zip {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let zip_token: keyword::zip = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression_1: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let flow_expression_2: Box<FlowExpression> = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(Zip {
                zip_token,
                paren_token,
                flow_expression_1,
                comma_token,
                flow_expression_2,
            })
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}

/// Component call.
#[derive(Debug, PartialEq, Clone)]
pub struct ComponentCall {
    /// Identifier to the component to call.
    ident_component: syn::Path,
    paren_token: token::Paren,
    /// Input expressions.
    inputs: Punctuated<FlowExpression, Token![,]>,
    /// Identifier to the component output signal to call.
    ident_signal: Option<(Token![.], syn::Ident)>,
}
impl Parse for ComponentCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_component: syn::Path = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let inputs: Punctuated<FlowExpression, Token![,]> = Punctuated::parse_terminated(&content)?;
        let ident_signal: Option<(Token![.], syn::Ident)> = {
            if input.peek(Token![.]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            }
        };
        Ok(ComponentCall {
            ident_component,
            paren_token,
            inputs,
            ident_signal,
        })
    }
}

/// Flow expression kinds.
#[derive(Debug, PartialEq, Clone)]
pub enum FlowExpression {
    /// GReact `tiemout` operator.
    Sample(Sample),
    /// GReact `map` operator.
    Map(Map),
    /// GReact `merge` operator.
    Merge(Merge),
    /// GReact `zip` operator.
    Zip(Zip),
    /// Component call.
    ComponentCall(ComponentCall),
    /// Another Rust expression.
    RustExpr(syn::Expr),
}
impl Parse for FlowExpression {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if Sample::peek(input) {
            Ok(FlowExpression::Sample(input.parse()?))
        } else if Map::peek(input) {
            Ok(FlowExpression::Map(input.parse()?))
        } else if Merge::peek(input) {
            Ok(FlowExpression::Merge(input.parse()?))
        } else if Zip::peek(input) {
            Ok(FlowExpression::Zip(input.parse()?))
        } else if input.fork().call(ComponentCall::parse).is_ok() {
            Ok(FlowExpression::ComponentCall(input.parse()?))
        } else {
            Ok(FlowExpression::RustExpr(input.parse()?))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlowKind {
    Signal(keyword::signal),
    Event(keyword::event),
}
impl FlowKind {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::signal) || input.peek(keyword::event)
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Flow statement AST.
pub struct FlowDeclaration {
    pub let_token: Token![let],
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_ident: (syn::Ident, Token![:], Type),
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}

#[derive(Debug, PartialEq, Clone)]
/// Flow statement AST.
pub struct FlowInstanciation {
    /// Identifier of the flow.
    pub ident: syn::Ident,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}

#[derive(Debug, PartialEq, Clone)]
/// Flow statement AST.
pub struct FlowImport {
    pub import_token: keyword::import,
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_ident: (syn::Ident, Token![:], Type),
    pub semi_token: Token![;],
}

#[derive(Debug, PartialEq, Clone)]
/// Flow statement AST.
pub struct FlowExport {
    pub export_token: keyword::export,
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_ident: (syn::Ident, Token![:], Type),
    pub semi_token: Token![;],
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instanciation(FlowInstanciation),
    Import(FlowImport),
    Export(FlowExport),
}
