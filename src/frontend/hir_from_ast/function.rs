use crate::ast::function::Function;
use crate::error::{Error, TerminationError};
use crate::hir::function::Function as HIRFunction;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Function {
    type HIR = HIRFunction;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Function {
            id,
            inputs,
            statements,
            returned: (_, returned),
            location,
        } = self;

        // create local context with all inputs
        symbol_table.local();
        let inputs = inputs
            .into_iter()
            .map(|(name, typing)| {
                let id = symbol_table.insert_identifier(
                    name,
                    Some(typing),
                    true,
                    location.clone(),
                    errors,
                )?;
                Ok(id)
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let id = symbol_table.get_function_id(&id, false, location.clone(), errors)?;

        let statements = statements
            .into_iter()
            .map(|statement| statement.hir_from_ast(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let returned = returned.hir_from_ast(symbol_table, errors)?;
        symbol_table.global();

        Ok(HIRFunction {
            id,
            inputs,
            statements,
            returned,
            location,
        })
    }
}
