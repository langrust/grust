//! HIR [Function](crate::hir::function::Function) module.

prelude! {}

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: usize,
    /// Function's contract.
    pub contract: hir::Contract,
    /// Function's statements.
    pub statements: Vec<hir::Stmt<hir::Expr>>,
    /// Function's returned expression and its type.
    pub returned: hir::Expr,
    /// Function location.
    pub location: Location,
}
