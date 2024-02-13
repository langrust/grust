use crate::ast::contract::Contract;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::contract::term::hir_from_ast as term_hir_from_ast;
use crate::hir::contract::Contract as HIRContract;
use crate::symbol_table::SymbolTable;

/// Transform AST contract into HIR contract.
pub fn hir_from_ast(
    contract: Contract,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRContract, TerminationError> {
    let Contract {
        requires,
        ensures,
        invariant,
    } = contract;

    Ok(HIRContract {
        requires: requires
            .into_iter()
            .map(|term| term_hir_from_ast(term, symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        ensures: ensures
            .into_iter()
            .map(|term| term_hir_from_ast(term, symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        invariant: invariant
            .into_iter()
            .map(|term| term_hir_from_ast(term, symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
    })
}

mod term {
    use crate::ast::contract::{Term, TermKind};
    use crate::common::location::Location;
    use crate::error::{Error, TerminationError};
    use crate::hir::contract::{Term as HIRTerm, TermKind as HIRTermKind};
    use crate::symbol_table::SymbolTable;

    pub fn hir_from_ast(
        term: Term,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRTerm, TerminationError> {
        let Term { kind, location } = term;
        match kind {
            TermKind::Binary { op, left, right } => Ok(HIRTerm {
                kind: HIRTermKind::Binary {
                    op,
                    left: Box::new(hir_from_ast(*left, symbol_table, errors)?),
                    right: Box::new(hir_from_ast(*right, symbol_table, errors)?),
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
