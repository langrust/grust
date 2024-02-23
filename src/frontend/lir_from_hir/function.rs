use itertools::Itertools;

use crate::{
    hir::{function::Function, identifier_creator::IdentifierCreator},
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

        // get function name
        let name = symbol_table.get_name(&id).clone();

        // get function inputs
        let mut inputs = symbol_table
            .get_function_input(&id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(id).clone(),
                    symbol_table.get_type(id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get function output type
        let mut output = symbol_table.get_function_output_type(&id).clone();

        // collect imports from statements, inputs and output types and returned expression
        let mut imports = statements
            .iter()
            .flat_map(|equation| equation.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut inputs_type_imports = inputs
            .iter()
            .flat_map(|(_, typing)| typing.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut output_type_imports = output.get_imports(symbol_table);
        let mut expression_imports = returned.get_imports(symbol_table);

        // combining all imports, eliminate duplicates and filter function imports
        imports.append(&mut inputs_type_imports);
        imports.append(&mut output_type_imports);
        imports.append(&mut expression_imports);
        let imports = imports
            .into_iter()
            .unique()
            .filter(|import| match import {
                Import::Enumeration(_)
                | Import::Structure(_)
                | Import::ArrayAlias(_)
                | Import::Creusot(_) => true,
                Import::Function(_) => false,
                Import::NodeFile(_) => unreachable!(),
            })
            .collect::<Vec<_>>();

        // get input's generics: function types in inputs
        let mut identifier_creator = IdentifierCreator::from(vec![]);
        let mut generics = inputs
            .iter_mut()
            .flat_map(|(_, typing)| typing.get_generics(&mut identifier_creator))
            .collect::<Vec<_>>();
        let mut output_generics = output.get_generics(&mut identifier_creator);
        generics.append(&mut output_generics);

        // tranforms into LIR statements
        let mut statements = statements
            .into_iter()
            .map(|statement| statement.lir_from_hir(symbol_table))
            .collect::<Vec<_>>();
        statements.push(Statement::ExpressionLast {
            expression: returned.lir_from_hir(symbol_table),
        });

        LIRFunction {
            name,
            generics,
            inputs,
            output,
            body: Block { statements },
            imports,
        }
    }
}
