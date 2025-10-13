use crate::{common::r#type::Type, lir::item::import::Import, symbol_table::SymbolTable};

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

/// LIR contract construction from HIR contract.
pub mod contract;

/// LIR memory construction from HIR typedef.
pub mod memory;

/// LIR pattern construction from HIR pattern.
pub mod pattern;

/// LIR type construction from HIR type.
pub mod r#type;

/// HIR transformation into LIR.
pub trait LIRFromHIR {
    /// Corresponding LIR construct.
    type LIR;

    /// Transforms HIR into LIR.
    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR;

    /// Get type from LIR.
    fn get_type(&self) -> Option<&Type> {
        None
    }

    /// Tell if LIR construct is an IfThenElse operator.
    fn is_if_then_else(&self, _symbol_table: &SymbolTable) -> bool {
        false
    }

    /// Get imports from LIR.
    fn get_imports(&self, _symbol_table: &SymbolTable) -> Vec<Import> {
        vec![]
    }
}
