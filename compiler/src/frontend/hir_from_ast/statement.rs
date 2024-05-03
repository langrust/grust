use crate::ast::expression::Expression;
use crate::ast::ident_colon::IdentColon;
use crate::ast::statement::LetDeclaration;
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression as HIRExpression, statement::Statement as HIRStatement};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for LetDeclaration<Expression> {
    type HIR = HIRStatement<HIRExpression>;

    // precondition: NOTHING is in symbol table
    // postcondition: construct HIR statement and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let LetDeclaration {
            typed_ident:
                IdentColon {
                    ident,
                    elem: element_type,
                    ..
                },
            expression,
            ..
        } = self;
        let location = Location::default();
        let element_name = ident.to_string();

        let typing = element_type.hir_from_ast(&location, symbol_table, errors)?;
        let id = symbol_table.insert_identifier(
            element_name,
            Some(typing),
            true,
            location.clone(),
            errors,
        )?;

        Ok(HIRStatement {
            id,
            expression: expression.hir_from_ast(symbol_table, errors)?,
            location,
        })
    }
}
