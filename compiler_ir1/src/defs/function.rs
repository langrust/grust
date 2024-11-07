//! [Function] module.

prelude! {}

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: usize,
    /// Function's contract.
    pub contract: ir1::Contract,
    /// Function's statements.
    pub statements: Vec<ir1::Stmt<ir1::Expr>>,
    /// Function's returned expression and its type.
    pub returned: ir1::Expr,
    /// Function location.
    pub loc: Location,
}
