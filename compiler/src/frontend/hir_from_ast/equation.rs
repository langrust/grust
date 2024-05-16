use crate::ast::equation::{Equation, Instanciation};
use crate::ast::statement::LetDeclaration;
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::{
    statement::Statement as HIRStatement,
    stream_expression::StreamExpression as HIRStreamExpression,
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Equation {
    type HIR = HIRStatement<HIRStreamExpression>;

    // precondition: equation's signal is already stored in symbol table
    // postcondition: construct HIR equation and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let location = Location::default();
        match self {
            Equation::LocalDef(LetDeclaration {
                typed_pattern: pattern,
                expression,
                ..
            })
            | Equation::OutputDef(Instanciation {
                pattern,
                expression,
                ..
            }) => {
                let typed_pattern = pattern.hir_from_ast(symbol_table, errors)?;

                Ok(HIRStatement {
                    typed_pattern,
                    expression: expression.hir_from_ast(symbol_table, errors)?,
                    location,
                })
            }
        }
    }
}
