use crate::ast::function::Function;
use crate::ast::ident_colon::IdentColon;
use crate::ast::statement::{ReturnInstruction, Statement};
use crate::common::location::Location;
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
            ident,
            output_type,
            statements,
            ..
        } = self;
        let name = ident.to_string();
        let location = Location::default();
        let id = symbol_table.get_function_id(&name, false, location.clone(), errors)?;

        // insert function output type in symbol table
        let output_typing = output_type.hir_from_ast(&location, symbol_table, errors)?;
        symbol_table.set_function_output_type(id, output_typing);

        // create local context with all signals
        symbol_table.local();
        symbol_table.restore_context(id);

        let (statements, returned) = statements.into_iter().fold(
            (vec![], None),
            |(mut declarations, option_returned), statement| match statement {
                Statement::Declaration(declaration) => {
                    declarations.push(declaration.hir_from_ast(symbol_table, errors));
                    (declarations, option_returned)
                }
                Statement::Return(ReturnInstruction { expression, .. }) => {
                    assert!(option_returned.is_none());
                    (
                        declarations,
                        Some(expression.hir_from_ast(symbol_table, errors)),
                    )
                }
            },
        );

        symbol_table.global();

        Ok(HIRFunction {
            id,
            statements: statements.into_iter().collect::<Result<Vec<_>, _>>()?,
            returned: returned.unwrap()?,
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

        let location = Location::default();
        let inputs = self
            .args
            .iter()
            .map(
                |IdentColon {
                     ident,
                     elem: typing,
                     ..
                 }| {
                    let name = ident.to_string();
                    let typing = typing
                        .clone()
                        .hir_from_ast(&location, symbol_table, errors)?;
                    let id = symbol_table.insert_identifier(
                        name.clone(),
                        Some(typing),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    Ok(id)
                },
            )
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        symbol_table.global();

        let _ = symbol_table.insert_function(
            self.ident.to_string(),
            inputs,
            None,
            false,
            location,
            errors,
        )?;

        Ok(())
    }
}
