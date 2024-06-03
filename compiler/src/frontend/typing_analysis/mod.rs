prelude! {}

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

/// LanGRust [Interface](crate::hir::interface::Interface) typing analysis module.
pub mod flow_expression;

pub mod flow_statement;

/// Performs type analysis.
pub trait TypeAnalysis {
    /// Tries to type the given construct.
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;

    /// Get type from construct.
    fn get_type(&self) -> Option<&Typ> {
        None
    }

    /// Get mutable type from construct.
    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        None
    }
}
