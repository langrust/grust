use crate::ast::equation::{Equation, Instanciation};
use crate::ast::ident_colon::IdentColon;
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
                typed_ident: IdentColon { ident, .. },
                expression,
                ..
            })
            | Equation::OutputDef(Instanciation {
                ident, expression, ..
            }) => {
                let name = ident.to_string();
                let id = symbol_table.get_signal_id(&name, true, location.clone(), errors)?;

                Ok(HIRStatement {
                    id,
                    expression: expression.hir_from_ast(symbol_table, errors)?,
                    location,
                })
            }
        }
    }
}
