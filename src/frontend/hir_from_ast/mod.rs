use crate::{
    error::{Error, TerminationError},
    symbol_table::SymbolTable,
};

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
/// HIR Node construction from AST Node
pub mod node;
/// HIR Pattern construction from AST Pattern
pub mod pattern;
/// HIR Statement construction from AST Statement
pub mod statement;
/// HIR StreamExpression construction from AST StreamExpression
pub mod stream_expression;
pub mod typedef;

pub trait HIRFromAST {
    type HIR;

    /// Transforms AST into HIR.
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError>;
}
