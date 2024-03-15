use crate::{
    common::r#type::Type,
    error::{Error, TerminationError},
    symbol_table::SymbolTable,
};

/// LanGRust [File](crate::hir::file::File) typing analysis module.
pub mod file;

/// LanGRust [Node](crate::hir::node::Node) typing analysis module.
pub mod node;

/// LanGRust [Function](crate::hir::function::Function) typing analysis module.
pub mod function;

/// LanGRust [StreamExpression](crate::hir::stream_expression::StreamExpression) typing analysis module.
pub mod stream_expression;

/// LanGRust [Expression](crate::hir::expression::Expression) typing analysis module.
pub mod expression;

/// LanGRust [Statement](crate::hir::statement::Statement) typing analysis module.
pub mod statement;

/// LanGRust [Pattern](crate::hir::pattern::Pattern) typing analysis module.
pub mod pattern;

/// Performs type analysis.
pub trait TypeAnalysis {
    /// Tries to type the given construct.
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError>;

    /// Get type from construct.
    fn get_type(&self) -> Option<&Type> {
        None
    }

    /// Get mutable type from construct.
    fn get_type_mut(&mut self) -> Option<&mut Type> {
        None
    }
}
