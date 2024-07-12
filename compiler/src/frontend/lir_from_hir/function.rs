prelude! {
    hir::Function,
    lir::{ Block, item::Function as LIRFunction, Stmt },
}

use super::LIRFromHIR;

impl LIRFromHIR for Function {
    type LIR = LIRFunction;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let Function {
            id,
            contract,
            statements,
            returned,
            ..
        } = self;

        // get function name
        let name = symbol_table.get_name(id).clone();

        // get function inputs
        let inputs = symbol_table
            .get_function_input(id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_type(*id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get function output type
        let output = symbol_table.get_function_output_type(id).clone();

        // tranforms into LIR statements
        let mut statements = statements
            .into_iter()
            .map(|statement| statement.lir_from_hir(symbol_table))
            .collect::<Vec<_>>();
        statements.push(Stmt::ExprLast {
            expression: returned.lir_from_hir(symbol_table),
        });

        // transform contract
        let contract = contract.lir_from_hir(symbol_table);

        LIRFunction {
            name,
            inputs,
            output,
            body: Block { statements },
            contract,
        }
    }
}
