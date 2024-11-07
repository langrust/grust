//! LIR construction from HIR.

prelude! {}

/// LIR node files construction from HIR node.
mod component;
/// LIR contract construction from HIR contract.
mod contract;
/// LIR expression construction from HIR expression.
mod expression;
/// LIR file construction from HIR project.
mod file;
/// LIR function construction from HIR function.
mod function;
mod interface;
/// LIR pattern construction from HIR pattern.
mod pattern;
/// LIR statement construction from HIR statement.
mod statement;
/// LIR expression construction from HIR stream expression.
mod stream_expression;
/// LIR item construction from HIR typedef.
mod typedef;

/// HIR transformation into LIR.
pub trait IntoLir<Ctx> {
    /// Corresponding LIR construct.
    type Lir;

    /// Transforms HIR into LIR.
    fn into_lir(self, ctx: Ctx) -> Self::Lir;

    /// Get type from LIR.
    fn get_type(&self) -> Option<&Typ> {
        None
    }

    /// Tell if LIR construct is an IfThenElse operator.
    fn is_if_then_else(&self, _symbol_table: &SymbolTable) -> bool {
        false
    }
}
