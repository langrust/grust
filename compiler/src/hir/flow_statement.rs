use syn::Token;

use crate::{ast::keyword, common::r#type::Type};

use super::{flow_expression::FlowExpression, pattern::Pattern};

/// Flow statement HIR.
pub struct FlowDeclaration {
    pub let_token: Token![let],
    /// Pattern of flows and their types.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
/// Flow statement HIR.
pub struct FlowInstanciation {
    /// Pattern of flows and their types.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
/// Flow statement HIR.
pub struct FlowImport {
    pub import_token: keyword::import,
    /// Identifier of the flow and its type.
    pub id: usize,
    pub path: syn::Path,
    pub colon_token: Token![:],
    pub flow_type: Type,
    pub semi_token: Token![;],
}
/// Flow statement HIR.
pub struct FlowExport {
    pub export_token: keyword::export,
    /// Identifier of the flow and its type.
    pub id: usize,
    pub path: syn::Path,
    pub colon_token: Token![:],
    pub flow_type: Type,
    pub semi_token: Token![;],
}

pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instanciation(FlowInstanciation),
    Import(FlowImport),
    Export(FlowExport),
}
