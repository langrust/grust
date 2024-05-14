use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::file::File;
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for File {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let File {
            functions,
            nodes,
            interface,
            ..
        } = self;

        // typing nodes
        nodes
            .iter_mut()
            .map(|node| node.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // typing functions
        functions
            .iter_mut()
            .map(|function| function.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // typing interface
        interface
            .iter_mut()
            .map(|statement| statement.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()
    }
}
