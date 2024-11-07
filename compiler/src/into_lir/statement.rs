prelude! { hir::Stmt }

impl<'a, E> IntoLir<&'a SymbolTable> for Stmt<E>
where
    E: IntoLir<&'a SymbolTable, Lir = lir::Expr>,
{
    type Lir = lir::Stmt;

    fn into_lir(self, symbol_table: &'a SymbolTable) -> Self::Lir {
        lir::Stmt::Let {
            pattern: self.pattern.into_lir(symbol_table),
            expr: self.expr.into_lir(symbol_table),
        }
    }
}
