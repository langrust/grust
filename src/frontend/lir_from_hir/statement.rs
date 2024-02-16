use crate::{
    hir::statement::Statement,
    lir::{expression::Expression as LIRExpression, statement::Statement as LIRStatement},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl<E> LIRFromHIR for Statement<E>
where
    E: LIRFromHIR<LIR = LIRExpression>,
{
    type LIR = LIRStatement;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let Statement { id, expression, .. } = self;
        LIRStatement::Let {
            identifier: symbol_table.get_name(&id).clone(),
            expression: expression.lir_from_hir(symbol_table),
        }
    }
}
