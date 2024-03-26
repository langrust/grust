use crate::ast::interface::Interface;
use crate::error::{Error, TerminationError};
use crate::hir::interface::Interface as HIRInterface;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Interface {
    type HIR = HIRInterface;

    // precondition: interface and its imports/exports are already stored in symbol table
    // postcondition: construct HIR interface and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        todo!()
    }
}

impl Interface {
    /// Store interface's identifiers in symbol table.
    pub fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let imports = self
            .imports
            .iter()
            .map(|(ty, path)| todo!("should I get id or create them?"))
            .collect();
        let exports = self
            .exports
            .iter()
            .map(|path| todo!("should I get id or create them?"))
            .collect();

        let _ = symbol_table.insert_interface(
            self.id.clone(),
            false,
            imports,
            exports,
            self.location.clone(),
            errors,
        )?;

        Ok(())
    }
}
