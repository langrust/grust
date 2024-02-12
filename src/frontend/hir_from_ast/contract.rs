use std::collections::HashMap;

use crate::ast::contract::Contract;
use crate::common::scope::Scope;
use crate::frontend::hir_from_ast::contract::term::hir_from_ast as term_hir_from_ast;
use crate::hir::contract::Contract as HIRContract;

/// Transform AST contract into HIR contract.
pub fn hir_from_ast(contract: Contract, signals_context: &HashMap<String, Scope>) -> HIRContract {
    let Contract {
        requires,
        ensures,
        invariant,
    } = contract;

    HIRContract {
        requires: requires
            .into_iter()
            .map(|term| term_hir_from_ast(term, signals_context))
            .collect(),
        ensures: ensures
            .into_iter()
            .map(|term| term_hir_from_ast(term, signals_context))
            .collect(),
        invariant: invariant
            .into_iter()
            .map(|term| term_hir_from_ast(term, signals_context))
            .collect(),
    }
}

mod term {
    use std::collections::HashMap;

    use crate::ast::contract::{Term, TermKind};
    use crate::common::scope::Scope;
    use crate::hir::contract::{Term as HIRTerm, TermKind as HIRTermKind};
    use crate::hir::signal::Signal;

    pub fn hir_from_ast(term: Term, signals_context: &HashMap<String, Scope>) -> HIRTerm {
        let Term { kind, location } = term;
        match kind {
            TermKind::Binary { op, left, right } => HIRTerm {
                kind: HIRTermKind::Binary {
                    op,
                    left: Box::new(hir_from_ast(*left, signals_context)),
                    right: Box::new(hir_from_ast(*right, signals_context)),
                },
                location,
            },
            TermKind::Constant { constant } => HIRTerm {
                kind: HIRTermKind::Constant { constant },
                location,
            },
            TermKind::Identifier { id } => {
                let scope = signals_context.get(&id).unwrap().clone();
                HIRTerm {
                    kind: HIRTermKind::Identifier {
                        signal: Signal { id, scope },
                    },
                    location,
                }
            }
        }
    }
}
