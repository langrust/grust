use itertools::Itertools;

use crate::{
    hir::function::Function,
    lir::{
        block::Block,
        item::{function::Function as LIRFunction, import::Import},
        statement::Statement,
    },
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Function {
    type LIR = LIRFunction;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let Function {
            id,
            statements,
            returned,
            ..
        } = self;

        // collect imports from statements
        let mut imports = statements
            .iter()
            .flat_map(|equation| equation.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut expression_imports = returned.get_imports(symbol_table);

        // combining both imports, eliminate duplicates and filter function imports
        imports.append(&mut expression_imports);
        let imports = imports
            .into_iter()
            .unique()
            .filter(|import| match import {
                Import::Enumeration(_) | Import::Structure(_) | Import::ArrayAlias(_) => true,
                Import::Function(_) => false,
                Import::NodeFile(_) => unreachable!(),
            })
            .collect::<Vec<_>>();

        // tranforms into LIR statements
        let mut statements = statements
            .into_iter()
            .map(|statement| statement.lir_from_hir(symbol_table))
            .collect::<Vec<_>>();
        statements.push(Statement::ExpressionLast {
            expression: returned.lir_from_hir(symbol_table),
        });

        // get inputs
        let inputs = symbol_table
            .get_function_input(&id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(id).clone(),
                    symbol_table.get_type(id).clone(),
                )
            })
            .collect();

        LIRFunction {
            name: symbol_table.get_name(&id).clone(),
            inputs,
            output: symbol_table.get_function_output_type(&id).clone(),
            body: Block { statements },
            imports,
        }
    }
}
