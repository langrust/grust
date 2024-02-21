use crate::{
    common::r#type::Type, lir::item::import::Import, symbol_table::SymbolTable,
};

/// LIR file construction from HIR project.
pub mod file;

/// LIR function construction from HIR function.
pub mod function;

/// LIR node files construction from HIR node.
pub mod node;

/// LIR expression construction from HIR expression.
pub mod expression;

/// LIR statement construction from HIR statement.
pub mod statement;

/// LIR expression construction from HIR stream expression.
pub mod stream_expression;

/// LIR node file construction from HIR unitary node.
pub mod unitary_node;

/// LIR item construction from HIR typedef.
pub mod typedef;

pub mod contract;
/// LIR memory construction from HIR typedef.
pub mod memory;
pub mod pattern;
pub mod r#type;

pub trait LIRFromHIR {
    type LIR;

    /// Transforms HIR into LIR.
    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR;

    fn get_type(&self) -> Option<&Type> {
        None
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        false
    }
    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        vec![]
    }
}
