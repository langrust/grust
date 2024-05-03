use syn::{parenthesized, parse::Parse, punctuated::Punctuated, token, Token};

use super::{
    ident_colon::{IdentColon, PathColon},
    keyword,
};
use crate::common::r#type::Type;

/// GReact `sample` operator.
pub struct Sample {
    pub sample_token: keyword::sample,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Sampling period in milliseconds.
    pub period_ms: syn::LitInt,
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

/// GReact `merge` operator.
pub struct Merge {
    pub merge_token: keyword::merge,
    pub paren_token: token::Paren,
    /// Input expression 1.
    pub flow_expression_1: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Input expression 2.
    pub flow_expression_2: Box<FlowExpression>,
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
pub struct Zip {
    pub zip_token: keyword::zip,
    pub paren_token: token::Paren,
    /// Input expression 1.
    pub flow_expression_1: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Input expression 2.
    pub flow_expression_2: Box<FlowExpression>,
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
pub struct ComponentCall {
    /// Identifier to the component to call.
    pub ident_component: syn::Ident,
    pub paren_token: token::Paren,
    /// Input expressions.
    pub inputs: Punctuated<FlowExpression, Token![,]>,
    /// Identifier to the component output signal to call.
    pub ident_signal: Option<(Token![.], syn::Ident)>,
}
impl Parse for ComponentCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_component: syn::Ident = input.parse()?;
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
pub enum FlowExpression {
    /// GReact `tiemout` operator.
    Sample(Sample),
    /// GReact `merge` operator.
    Merge(Merge),
    /// GReact `zip` operator.
    Zip(Zip),
    /// Component call.
    ComponentCall(ComponentCall),
    /// Another Rust expression.
    Ident(syn::Ident),
}
impl Parse for FlowExpression {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if Sample::peek(input) {
            Ok(FlowExpression::Sample(input.parse()?))
        } else if Merge::peek(input) {
            Ok(FlowExpression::Merge(input.parse()?))
        } else if Zip::peek(input) {
            Ok(FlowExpression::Zip(input.parse()?))
        } else if input.fork().call(ComponentCall::parse).is_ok() {
            Ok(FlowExpression::ComponentCall(input.parse()?))
        } else {
            Ok(FlowExpression::Ident(input.parse()?))
        }
    }
}

pub enum FlowKind {
    Signal(keyword::signal),
    Event(keyword::event),
}
impl FlowKind {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::signal) || input.peek(keyword::event)
    }
}
impl Parse for FlowKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::signal) {
            Ok(FlowKind::Signal(input.parse()?))
        } else if input.peek(keyword::event) {
            Ok(FlowKind::Event(input.parse()?))
        } else {
            Err(input.error("expected 'signal' or 'event'"))
        }
    }
}

/// Flow statement AST.
pub struct FlowDeclaration {
    pub let_token: Token![let],
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_ident: IdentColon<Type>,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
impl FlowDeclaration {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![let])
    }
}
impl Parse for FlowDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let let_token: Token![let] = input.parse()?;
        let kind: FlowKind = input.parse()?;
        let typed_ident: IdentColon<Type> = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let flow_expression: FlowExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowDeclaration {
            let_token,
            kind,
            typed_ident,
            eq_token,
            flow_expression,
            semi_token,
        })
    }
}

/// Flow statement AST.
pub struct FlowInstanciation {
    /// Identifier of the flow.
    pub ident: syn::Ident,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
impl FlowInstanciation {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(Token![=])
    }
}
impl Parse for FlowInstanciation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let flow_expression: FlowExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowInstanciation {
            ident,
            eq_token,
            flow_expression,
            semi_token,
        })
    }
}

/// Flow statement AST.
pub struct FlowImport {
    pub import_token: keyword::import,
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_path: PathColon<Type>,
    pub semi_token: Token![;],
}
impl FlowImport {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::import)
    }
}
impl Parse for FlowImport {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let import_token: keyword::import = input.parse()?;
        let kind: FlowKind = input.parse()?;
        let typed_path: PathColon<Type> = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowImport {
            import_token,
            kind,
            typed_path,
            semi_token,
        })
    }
}

/// Flow statement AST.
pub struct FlowExport {
    pub export_token: keyword::export,
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_path: PathColon<Type>,
    pub semi_token: Token![;],
}
impl FlowExport {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::export)
    }
}
impl Parse for FlowExport {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let export_token: keyword::export = input.parse()?;
        let kind: FlowKind = input.parse()?;
        let typed_path: PathColon<Type> = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowExport {
            export_token,
            kind,
            typed_path,
            semi_token,
        })
    }
}

pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instanciation(FlowInstanciation),
    Import(FlowImport),
    Export(FlowExport),
}
impl FlowStatement {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        FlowDeclaration::peek(input)
            || FlowInstanciation::peek(input)
            || FlowImport::peek(input)
            || FlowExport::peek(input)
    }
}
impl Parse for FlowStatement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if FlowDeclaration::peek(input) {
            Ok(FlowStatement::Declaration(input.parse()?))
        } else if FlowInstanciation::peek(input) {
            Ok(FlowStatement::Instanciation(input.parse()?))
        } else if FlowImport::peek(input) {
            Ok(FlowStatement::Import(input.parse()?))
        } else if FlowExport::peek(input) {
            Ok(FlowStatement::Export(input.parse()?))
        } else {
            Err(input.error("expected flow declaration, instanciation, import or export"))
        }
    }
}
