use syn::{punctuated::Punctuated, token, Token};

use super::keyword;
use crate::common::r#type::Type;

#[derive(Debug, PartialEq, Clone)]
pub enum FlowKind {
    Signal(keyword::signal),
    Event(keyword::event),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instanciation(FlowInstanciation),
    Import(FlowImport),
    Export(FlowExport),
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
/// Flow expression kinds.
pub enum FlowExpression {
    /// GReact `tiemout` operator.
    Timeout {
        timeout_token: keyword::timeout,
        paren_token: token::Paren,
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        comma_token: Token![,],
        /// Time of the timeout in milliseconds.
        timeout_ms: syn::LitInt,
    },
    /// GReact `map` operator.
    Map {
        map_token: keyword::map,
        paren_token: token::Paren,
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        comma_token: Token![,],
        /// Time of the timeout in milliseconds.
        function: syn::Expr,
    },
    /// GReact `merge` operator.
    Merge {
        merge_token: keyword::merge,
        paren_token: token::Paren,
        /// Input expression 1.
        flow_expression_1: Box<FlowExpression>,
        comma_token: Token![,],
        /// Input expression 2.
        flow_expression_2: Box<FlowExpression>,
    },
    /// GReact `zip` operator.
    Zip {
        zip_token: keyword::zip,
        paren_token: token::Paren,
        /// Input expression 1.
        flow_expression_1: Box<FlowExpression>,
        comma_token: Token![,],
        /// Input expression 2.
        flow_expression_2: Box<FlowExpression>,
    },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        ident_component: syn::Path,
        paren_token: token::Paren,
        /// Input expressions.
        inputs: Punctuated<FlowExpression, Token![,]>,
        /// Identifier to the component output signal to call.
        ident_signal: Option<(Token![.], syn::Ident)>,
    },
    RustExpr(syn::Expr),
}
