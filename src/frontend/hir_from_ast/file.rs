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
        let File {
            typedefs,
            functions,
            nodes,
            component,
            location,
        } = self;

        // TODO: this is supposed to be in another function in order to call nodes in any order
        // let inputs = inputs
        //     .into_iter()
        //     .map(|(name, typing)| {
        //         let id = symbol_table.insert_signal(name, Scope::Input, true, location, errors)?;
        //         // TODO: add type to signal in symbol table
        //         Ok(id)
        //     })
        //     .collect::<Vec<Result<_, _>>>()
        //     .into_iter()
        //     .collect::<Result<Vec<_>, _>>()?;
        // let outputs = equations
        //     .into_iter()
        //     .filter(|(name, equation)| Scope::Output == equation.scope)
        //     .map(|(name, equation)| {
        //         let id =
        //             symbol_table.insert_signal(name.clone(), Scope::Output, true, location, errors)?;
        //         // TODO: add type to signal in symbol table
        //         Ok((name, id))
        //     })
        //     .collect::<Vec<Result<_, _>>>()
        //     .into_iter()
        //     .collect::<Result<HashMap<_, _>, _>>()?;
        // let locals = equations
        //     .into_iter()
        //     .filter(|(name, equation)| Scope::Local == equation.scope)
        //     .map(|(name, equation)| {
        //         let id =
        //             symbol_table.insert_signal(name.clone(), Scope::Local, true, location, errors)?;
        //         // TODO: add type to signal in symbol table
        //         Ok((name, id))
        //     })
        //     .collect::<Vec<Result<_, _>>>()
        //     .into_iter()
        //     .collect::<Result<HashMap<_, _>, _>>()?;
        // let id = symbol_table.insert_node(
        //     id,
        //     is_component,
        //     false,
        //     inputs,
        //     outputs,
        //     locals,
        //     location,
        //     errors,
        // )?;

        // let id = symbol_table.insert_function(
        //     id,
        //     is_component,
        //     false,
        //     inputs,
        //     outputs,
        //     locals,
        //     location,
        //     errors,
        // )?;

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
            location,
        })
    }
}
