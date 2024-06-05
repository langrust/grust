//! HIR [Function](crate::hir::function::Function) module.

prelude! {
    hir::{Expr, Stmt, Contract}
}

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: usize,
    /// Function's contract.
    pub contract: Contract,
    /// Function's statements.
    pub statements: Vec<Stmt<Expr>>,
    /// Function's returned expression and its type.
    pub returned: Expr,
    /// Function location.
    pub location: Location,
}
