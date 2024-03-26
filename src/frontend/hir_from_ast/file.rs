use crate::ast::file::File;
use crate::error::{Error, TerminationError};
use crate::hir::file::File as HIRFile;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for File {
    type HIR = HIRFile;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        // initialize symbol table with builtin operators
        symbol_table.initialize();

        // store elements in symbol table
        self.store(symbol_table, errors)?;

        let File {
            typedefs,
            functions,
            nodes,
            component,
            interface,
            location,
        } = self;

        Ok(HIRFile {
            typedefs: typedefs
                .into_iter()
                .map(|typedef| typedef.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            functions: functions
                .into_iter()
                .map(|function| function.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            nodes: nodes
                .into_iter()
                .map(|node| node.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            component: component
                .map(|node| node.hir_from_ast(symbol_table, errors))
                .transpose()?,
            interface: todo!(),
            location,
        })
    }
}

impl File {
    fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        self.typedefs
            .iter()
            .map(|typedef| typedef.store(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        self.functions
            .iter()
            .map(|function| function.store(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        self.nodes
            .iter()
            .map(|node| node.store(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        self.component
            .as_ref()
            .map(|node| node.store(symbol_table, errors))
            .transpose()?;
        Ok(())
    }
}
