use crate::{
    hir::function::Function,
    lir::{block::Block, item::function::Function as LIRFunction, statement::Statement},
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

        // TODO: imports
        // let imports = equations
        //     .iter()
        //     .flat_map(|equation| equation.expression.get_imports())
        //     .unique()
        //     .collect();

        let mut statements = statements
            .into_iter()
            .map(|statement| statement.lir_from_hir(symbol_table))
            .collect::<Vec<_>>();
        statements.push(Statement::ExpressionLast {
            expression: returned.lir_from_hir(symbol_table),
        });

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
        }
    }
}
