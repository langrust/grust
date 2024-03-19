use crate::ast::function::Function;
use crate::error::{Error, TerminationError};
use crate::hir::function::Function as HIRFunction;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Function {
    type HIR = HIRFunction;

    // precondition: function and its inputs are already stored in symbol table
    // postcondition: construct HIR function and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Function {
            id,
            statements,
            returned: (_, returned),
            location,
            ..
        } = self;

        let id = symbol_table.get_function_id(&id, false, location.clone(), errors)?;

        // insert function output type in symbol table
        let output_typing = self
            .returned
            .0
            .hir_from_ast(&location, symbol_table, errors)?;
        symbol_table.set_function_output_type(id, output_typing);

        // create local context with all signals
        symbol_table.local();
        symbol_table.restore_context(id);

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
            statements,
            returned,
            location,
        })
    }
}

impl Function {
    /// Store function's identifiers in symbol table.
    pub fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        symbol_table.local();

        let inputs = self
            .inputs
            .iter()
            .map(|(name, typing)| {
                let typing = typing
                    .clone()
                    .hir_from_ast(&self.location, symbol_table, errors)?;
                let id = symbol_table.insert_identifier(
                    name.clone(),
                    Some(typing),
                    true,
                    self.location.clone(),
                    errors,
                )?;
                Ok(id)
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        symbol_table.global();

        let _ = symbol_table.insert_function(
            self.id.clone(),
            inputs,
            None,
            false,
            self.location.clone(),
            errors,
        )?;

        Ok(())
    }
}
