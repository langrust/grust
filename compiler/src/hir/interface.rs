use petgraph::graphmap::DiGraphMap;
use syn::Token;

use crate::{
    ast::{interface::FlowKind, keyword},
    common::r#type::Type,
    symbol_table::SymbolTable,
};

use super::{flow_expression::FlowExpression, pattern::Pattern};

pub struct Interface {
    pub statements: Vec<FlowStatement>,
    /// Flows dependency graph.
    pub graph: DiGraphMap<usize, FlowKind>,
}
impl Interface {
    pub fn get_flows_names(&self, symbol_table: &SymbolTable) -> Vec<String> {
        self.statements
            .iter()
            .flat_map(|statement| match statement {
                FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                | FlowStatement::Instanciation(FlowInstanciation { pattern, .. }) => pattern
                    .identifiers()
                    .into_iter()
                    .map(|id| symbol_table.get_name(id).clone())
                    .collect(),
                FlowStatement::Import(FlowImport { id, .. })
                | FlowStatement::Export(FlowExport { id, .. }) => {
                    vec![symbol_table.get_name(*id).clone()]
                }
            })
            .collect()
    }
    pub fn get_flows_ids(&self) -> Vec<usize> {
        self.statements
            .iter()
            .flat_map(|statement| match statement {
                FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                | FlowStatement::Instanciation(FlowInstanciation { pattern, .. }) => {
                    pattern.identifiers()
                }
                FlowStatement::Import(FlowImport { id, .. })
                | FlowStatement::Export(FlowExport { id, .. }) => {
                    vec![*id]
                }
            })
            .collect()
    }
}

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
