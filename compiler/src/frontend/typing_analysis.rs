//! Typing analysis from HIR.

prelude! {}

// "Empty" modules, *i.e.* only define `impl`-s and need not be `pub`lic.
mod component;
mod contract;
mod expression;
mod file;
mod flow_expression;
mod function;
mod interface;
mod pattern;
mod statement;
mod stream_expression;

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
