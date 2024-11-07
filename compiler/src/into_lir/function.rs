prelude! {
    hir::Function,
    lir::{ Block, item::Function as LIRFunction, Stmt },
}

impl IntoLir<&'_ SymbolTable> for Function {
    type Lir = LIRFunction;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        // get function name
        let name = symbol_table.get_name(self.id).clone();

        // get function inputs
        let inputs = symbol_table
            .get_function_input(self.id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_type(*id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get function output type
        let output = symbol_table.get_function_output_type(self.id).clone();

        // Transforms into LIR statements
        let mut statements = self
            .statements
            .into_iter()
            .map(|statement| statement.into_lir(symbol_table))
            .collect::<Vec<_>>();
        statements.push(Stmt::ExprLast {
            expr: self.returned.into_lir(symbol_table),
        });

        // transform contract
        let contract = self.contract.into_lir(symbol_table);

        LIRFunction {
            name,
            inputs,
            output,
            body: Block { statements },
            contract,
        }
    }
}
