prelude! {}

/// LIR file construction from HIR project.
pub mod file;

/// LIR function construction from HIR function.
pub mod function;

/// LIR node files construction from HIR node.
pub mod component;

/// LIR expression construction from HIR expression.
pub mod expression;

/// LIR statement construction from HIR statement.
pub mod statement;

/// LIR expression construction from HIR stream expression.
pub mod stream_expression;

/// LIR item construction from HIR typedef.
pub mod typedef;

/// LIR contract construction from HIR contract.
pub mod contract;

/// LIR memory construction from HIR typedef.
pub mod memory;

/// LIR pattern construction from HIR pattern.
pub mod pattern;

pub mod interface;

/// HIR transformation into LIR.
pub trait IntoLir<Ctx> {
    /// Corresponding LIR construct.
    type Lir;

    /// Transforms HIR into LIR.
    fn into_lir(self, symbol_table: Ctx) -> Self::Lir;

    /// Get type from LIR.
    fn get_type(&self) -> Option<&Typ> {
        None
    }

    /// Tell if LIR construct is an IfThenElse operator.
    fn is_if_then_else(&self, _symbol_table: &SymbolTable) -> bool {
        false
    }
}
