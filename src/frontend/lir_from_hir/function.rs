use crate::{
    hir::function::Function,
    lir::{block::Block, item::function::Function as LIRFunction, statement::Statement},
    symbol_table::SymbolTable,
};

use super::{
    expression::lir_from_hir as expression_lir_from_hir,
    statement::lir_from_hir as statement_lir_from_hir,
};
/// Transform HIR function into LIR function.
pub fn lir_from_hir(function: Function, symbol_table: &SymbolTable) -> LIRFunction {
    let Function {
        id,
        inputs,
        statements,
        returned,
        ..
    } = function;

    // TODO: imports
    // let imports = equations
    //     .iter()
    //     .flat_map(|equation| equation.expression.get_imports())
    //     .unique()
    //     .collect();
    
    let mut statements = statements
        .into_iter()
        .map(|statement| statement_lir_from_hir(statement, symbol_table))
        .collect::<Vec<_>>();
    statements.push(Statement::ExpressionLast {
        expression: expression_lir_from_hir(returned, symbol_table),
    });

    let inputs = inputs
        .into_iter()
        .map(|id| {
            (
                symbol_table.get_name(&id).clone(),
                symbol_table.get_type(&id).clone(),
            )
        })
        .collect();

    LIRFunction {
        name: symbol_table.get_name(&id).clone(),
        inputs,
        output: symbol_table.get_output_type(&id).clone(),
        body: Block { statements },
    }
}
