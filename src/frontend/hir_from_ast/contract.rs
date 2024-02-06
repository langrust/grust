use std::collections::HashMap;

use crate::ast::term::Contract;
use crate::common::scope::Scope;
use crate::hir::term::Contract as HIRContract;
use crate::frontend::hir_from_ast::contract::term::hir_from_ast as term_hir_from_ast;

pub fn hir_from_ast(contract: Contract, signals_context: &HashMap<String, Scope>) -> HIRContract {
    let Contract { requires, ensures, invariant, assert } = contract;


    HIRContract {
        requires: requires.into_iter().map(|term| term_hir_from_ast(term, signals_context)).collect(),
        ensures: ensures.into_iter().map(|term| term_hir_from_ast(term, signals_context)).collect(),
        invariant: invariant.into_iter().map(|term| term_hir_from_ast(term, signals_context)).collect(),
        assert: assert.into_iter().map(|term| term_hir_from_ast(term, signals_context)).collect(),
    }
}

mod term {
    use std::collections::HashMap;

    use crate::ast::term::{ Term, TermKind};
    use crate::common::scope::Scope;
    use crate::hir::signal::Signal;
    use crate::hir::term::{ Term as HIRTerm, TermKind as HIRTermKind};

    pub fn hir_from_ast(term: Term, signals_context: &HashMap<String, Scope>) -> HIRTerm {
        let Term {
            kind,
            location,
        } = term;
        match kind {
            TermKind::Binary { op, left, right } => {
                HIRTerm {
                    kind: HIRTermKind::Binary { 
                        op,
                        left: Box::new(hir_from_ast(*left, signals_context)),
                        right: Box::new(hir_from_ast(*right, signals_context)),
                    },
                    location,
                }
            },
            TermKind::Constant { constant } => {
                HIRTerm {
                    kind: HIRTermKind::Constant { 
                        constant
                    },
                    location,
                }
            },
            TermKind::Variable { id } => {
                let scope = signals_context.get(&id).unwrap().clone();
                HIRTerm {
                    kind: HIRTermKind::Variable { signal: Signal {id, scope} },
                    location,
                }
            },
        }
    }
}