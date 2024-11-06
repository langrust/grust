prelude! {}

/// HIR Component construction from AST Component
pub mod component;
/// HIR Contract construction from AST Contract
pub mod contract;
/// HIR Equation construction from AST Equation
pub mod equation;
/// HIR Expression construction from AST Expression
pub mod expression;
/// HIR File construction from AST File
pub mod file;
/// HIR Function construction from AST Function
pub mod function;
/// HIR Interface construction from AST Interface.
pub mod interface;
/// HIR Pattern construction from AST Pattern
pub mod pattern;
/// HIR Statement construction from AST Statement
pub mod statement;
/// HIR StreamExpression construction from AST StreamExpression
pub mod stream_expression;
pub mod typ;
/// HIR Typedef construction from AST Typedef.
pub mod typedef;

/// AST transformation into HIR.
pub trait IntoHir<Ctxt> {
    /// Corresponding HIR construct.
    type Hir;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut Ctxt) -> TRes<Self::Hir>;
}
