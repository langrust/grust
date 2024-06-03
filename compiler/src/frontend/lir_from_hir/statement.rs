prelude! {
    hir::Stmt,
    lir::item::Import,
}

use super::LIRFromHIR;

impl<E> LIRFromHIR for Stmt<E>
where
    E: LIRFromHIR<LIR = lir::Expr>,
{
    type LIR = lir::Stmt;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let Stmt {
            pattern,
            expression,
            ..
        } = self;
        lir::Stmt::Let {
            pattern: pattern.lir_from_hir(symbol_table),
            expression: expression.lir_from_hir(symbol_table),
        }
    }

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        self.expression.get_imports(symbol_table)
    }
}
