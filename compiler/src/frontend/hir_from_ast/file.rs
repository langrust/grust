use crate::ast::Ast;
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::file::File as HIRFile;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Ast {
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

        let Ast { items } = self;

        let (typedefs, functions, nodes, interface) = items.into_iter().fold(
            (vec![], vec![], vec![], vec![]),
            |(mut typedefs, mut functions, mut nodes, mut interface), item| match item {
                crate::ast::Item::Component(component) => {
                    nodes.push(component.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, interface)
                }
                crate::ast::Item::Function(function) => {
                    functions.push(function.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, interface)
                }
                crate::ast::Item::Typedef(typedef) => {
                    typedefs.push(typedef.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, interface)
                }
                crate::ast::Item::FlowStatement(_) => todo!(),
                crate::ast::Item::Rust(_) => todo!(),
            },
        );

        Ok(HIRFile {
            typedefs: typedefs.into_iter().collect::<Result<Vec<_>, _>>()?,
            functions: functions.into_iter().collect::<Result<Vec<_>, _>>()?,
            nodes: nodes.into_iter().collect::<Result<Vec<_>, _>>()?,
            component: None,
            interface: interface.into_iter().collect::<Result<Vec<_>, _>>()?,
            location: Location::default(),
        })
    }
}

impl Ast {
    fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        self.items
            .iter()
            .map(|item| match item {
                crate::ast::Item::Component(component) => component.store(symbol_table, errors),
                crate::ast::Item::Function(function) => function.store(symbol_table, errors),
                crate::ast::Item::Typedef(typedef) => typedef.store(symbol_table, errors),
                crate::ast::Item::FlowStatement(_) => todo!(),
                crate::ast::Item::Rust(_) => Ok(()),
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}
