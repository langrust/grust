use crate::ast::expression::Expression;
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
            typed_pattern,
            expression,
            ..
        } = self;
        let location = Location::default();
        typed_pattern.store(true, symbol_table, errors)?;
        let pattern = typed_pattern.hir_from_ast(symbol_table, errors)?;

        Ok(HIRStatement {
            pattern,
            expression: expression.hir_from_ast(symbol_table, errors)?,
            location,
        })
    }
}
