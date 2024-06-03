prelude! {
    ast::expr::*,
    hir::Dependencies,
}

use super::HIRFromAST;

impl HIRFromAST for Expr {
    type HIR = hir::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
        let loc = &location;
        let kind = match self {
            Self::Constant(constant) => hir::expr::Kind::Constant { constant },
            Self::Identifier(id) => {
                let id = symbol_table
                    .get_identifier_id(&id, false, location.clone(), &mut vec![])
                    .or_else(|_| {
                        symbol_table.get_function_id(&id, false, location.clone(), errors)
                    })?;
                hir::expr::Kind::Identifier { id }
            }
            Self::Unop(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Binop(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::IfThenElse(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Application(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::TypedAbstraction(expression) => {
                expression.hir_from_ast(loc, symbol_table, errors)?
            }
            Self::Structure(expression) => {
                expression.hir_from_ast(&location, symbol_table, errors)?
            }
            Self::Tuple(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Enumeration(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Array(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Match(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::FieldAccess(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::TupleElementAccess(expression) => {
                expression.hir_from_ast(loc, symbol_table, errors)?
            }
            Self::Map(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Fold(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Sort(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
            Self::Zip(expression) => expression.hir_from_ast(loc, symbol_table, errors)?,
        };
        Ok(hir::Expr {
            kind,
            typing: None,
            location,
            dependencies: Dependencies::new(),
        })
    }
}
