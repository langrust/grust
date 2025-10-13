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
            component,
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

        // typing component
        component
            .as_mut()
            .map_or(Ok(()), |component| component.typing(symbol_table, errors))?;

        // typing functions
        functions
            .iter_mut()
            .map(|function| function.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // typing interface
        interface
            .as_mut()
            .map_or(Ok(()), |interface| interface.typing(symbol_table, errors))
    }
}
