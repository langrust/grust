use crate::ast::contract::Contract;
use crate::error::{Error, TerminationError};
use crate::hir::contract::Contract as HIRContract;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Contract {
    type HIR = HIRContract;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Contract {
            requires,
            ensures,
            invariant,
        } = self;

        Ok(HIRContract {
            requires: requires
                .into_iter()
                .map(|term| term.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            ensures: ensures
                .into_iter()
                .map(|term| term.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            invariant: invariant
                .into_iter()
                .map(|term| term.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

mod term {
    use crate::ast::contract::{Term, TermKind};
    use crate::common::location::Location;
    use crate::error::{Error, TerminationError};
    use crate::hir::contract::{Term as HIRTerm, TermKind as HIRTermKind};
    use crate::symbol_table::SymbolTable;

    use super::HIRFromAST;

    impl HIRFromAST for Term {
        type HIR = HIRTerm;

        fn hir_from_ast(
            self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> Result<Self::HIR, TerminationError> {
            let Term { kind, location } = self;
            match kind {
                TermKind::Binary { op, left, right } => Ok(HIRTerm {
                    kind: HIRTermKind::Binary {
                        op,
                        left: Box::new(left.hir_from_ast(symbol_table, errors)?),
                        right: Box::new(right.hir_from_ast(symbol_table, errors)?),
                    },
                    location,
                }),
                TermKind::Constant { constant } => Ok(HIRTerm {
                    kind: HIRTermKind::Constant { constant },
                    location,
                }),
                TermKind::Identifier { id } => {
                    let id = symbol_table.get_signal_id(&id, true, Location::default(), errors)?;
                    Ok(HIRTerm {
                        kind: HIRTermKind::Identifier { id },
                        location,
                    })
                }
            }
        }
    }
}
