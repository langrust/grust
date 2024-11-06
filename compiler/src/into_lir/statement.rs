prelude! { hir::Stmt }

impl<'a, E> IntoLir<&'a SymbolTable> for Stmt<E>
where
    E: IntoLir<&'a SymbolTable, Lir = lir::Expr>,
{
    type Lir = lir::Stmt;

    fn into_lir(self, symbol_table: &'a SymbolTable) -> Self::Lir {
        let Stmt {
            pattern,
            expression,
            ..
        } = self;
        lir::Stmt::Let {
            pattern: pattern.into_lir(symbol_table),
            expression: expression.into_lir(symbol_table),
        }
    }
}
