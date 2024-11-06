//! HIR construction from AST.

prelude! {}

/// HIR Component construction from AST Component
mod component;
/// HIR Contract construction from AST Contract
mod contract;
/// HIR Equation construction from AST Equation
mod equation;
/// HIR Expression construction from AST Expression
mod expression;
/// HIR File construction from AST File
mod file;
/// HIR Function construction from AST Function
mod function;
/// HIR Interface construction from AST Interface.
mod interface;
/// HIR Pattern construction from AST Pattern
mod pattern;
/// HIR Statement construction from AST Statement
mod statement;
/// HIR StreamExpression construction from AST StreamExpression
mod stream_expression;
mod typ;
/// HIR Typedef construction from AST Typedef.
mod typedef;

/// AST transformation into HIR.
pub trait IntoHir<Ctxt> {
    /// Corresponding HIR construct.
    type Hir;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut Ctxt) -> TRes<Self::Hir>;
}
