use syn::{parenthesized, parse::Parse, punctuated::Punctuated, token, Token};

use super::{colon::Colon, keyword, pattern::Pattern};
use crate::common::{constant::Constant, r#type::Type};

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

/// GReact `scan` operator.
pub struct Scan {
    pub sample_token: keyword::scan,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Scaning period in milliseconds.
    pub period_ms: syn::LitInt,
}
impl Scan {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::scan)
    }
}
impl Parse for Scan {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sample_token: keyword::scan = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let period_ms: syn::LitInt = content.parse()?;
        if content.is_empty() {
            Ok(Scan {
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

/// GReact `timeout` operator.
pub struct Timeout {
    pub sample_token: keyword::timeout,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Deadline in milliseconds.
    pub deadline: syn::LitInt,
}
impl Timeout {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::timeout)
    }
}
impl Parse for Timeout {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sample_token: keyword::timeout = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let deadline: syn::LitInt = content.parse()?;
        if content.is_empty() {
            Ok(Timeout {
                sample_token,
                paren_token,
                flow_expression,
                comma_token,
                deadline,
            })
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}

/// GReact `throtle` operator.
pub struct Throtle {
    pub sample_token: keyword::throtle,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Variation that will update the signal.
    pub delta: Constant,
}
impl Throtle {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::throtle)
    }
}
impl Parse for Throtle {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sample_token: keyword::throtle = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: Box<FlowExpression> = Box::new(content.parse()?);
        let comma_token: Token![,] = content.parse()?;
        let delta: Constant = content.parse()?;
        if content.is_empty() {
            Ok(Throtle {
                sample_token,
                paren_token,
                flow_expression,
                comma_token,
                delta,
            })
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}

/// GReact `on_change` operator.
pub struct OnChange {
    pub sample_token: keyword::on_change,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
}
impl OnChange {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::on_change)
    }
}
impl Parse for OnChange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sample_token: keyword::on_change = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: Box<FlowExpression> = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(OnChange {
                sample_token,
                paren_token,
                flow_expression,
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
}
impl Parse for ComponentCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_component: syn::Ident = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let inputs: Punctuated<FlowExpression, Token![,]> = Punctuated::parse_terminated(&content)?;
        // let ident_signal: Option<(Token![.], syn::Ident)> = {
        //     if input.peek(Token![.]) {
        //         Some((input.parse()?, input.parse()?))
        //     } else {
        //         None
        //     }
        // };
        Ok(ComponentCall {
            ident_component,
            paren_token,
            inputs,
        })
    }
}

/// Flow expression kinds.
pub enum FlowExpression {
    /// GReact `sample` operator.
    Sample(Sample),
    /// GReact `scan` operator.
    Scan(Scan),
    /// GReact `timeout` operator.
    Timeout(Timeout),
    /// GReact `throtle` operator.
    Throtle(Throtle),
    /// GReact `on_change` operator.
    OnChange(OnChange),
    /// Component call.
    ComponentCall(ComponentCall),
    /// Identifier to flow.
    Ident(syn::Ident),
}
impl Parse for FlowExpression {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if Sample::peek(input) {
            Ok(FlowExpression::Sample(input.parse()?))
        } else if Scan::peek(input) {
            Ok(FlowExpression::Scan(input.parse()?))
        } else if Timeout::peek(input) {
            Ok(FlowExpression::Timeout(input.parse()?))
        } else if Throtle::peek(input) {
            Ok(FlowExpression::Throtle(input.parse()?))
        } else if OnChange::peek(input) {
            Ok(FlowExpression::OnChange(input.parse()?))
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
    /// Pattern of declared flows and their type.
    pub typed_pattern: Pattern,
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
        let typed_pattern: Pattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let flow_expression: FlowExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowDeclaration {
            let_token,
            kind,
            typed_pattern,
            eq_token,
            flow_expression,
            semi_token,
        })
    }
}

/// Flow statement AST.
pub struct FlowInstanciation {
    /// Pattern of instanciated flows.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
impl FlowInstanciation {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(Pattern::parse).is_err() {
            return false;
        }
        forked.peek(Token![=])
    }
}
impl Parse for FlowInstanciation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern: Pattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let flow_expression: FlowExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowInstanciation {
            pattern,
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
    pub typed_path: Colon<syn::Path, Type>,
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
        let typed_path: Colon<syn::Path, Type> = input.parse()?;
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
    pub typed_path: Colon<syn::Path, Type>,
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
        let typed_path: Colon<syn::Path, Type> = input.parse()?;
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
            || FlowImport::peek(input)
            || FlowExport::peek(input)
            || FlowInstanciation::peek(input)
    }
}
impl Parse for FlowStatement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if FlowDeclaration::peek(input) {
            Ok(FlowStatement::Declaration(input.parse()?))
        } else if FlowImport::peek(input) {
            Ok(FlowStatement::Import(input.parse()?))
        } else if FlowExport::peek(input) {
            Ok(FlowStatement::Export(input.parse()?))
        } else if FlowInstanciation::peek(input) {
            Ok(FlowStatement::Instanciation(input.parse()?))
        } else {
            Err(input.error("expected flow declaration, instanciation, import or export"))
        }
    }
}
