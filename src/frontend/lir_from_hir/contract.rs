use crate::{
    frontend::lir_from_hir::contract::term::lir_from_hir as term_lir_from_hir,
    hir::contract::Contract, lir::contract::Contract as LIRContract, symbol_table::SymbolTable,
};

/// Transform HIR contract into LIR contract.
pub fn lir_from_hir(contract: Contract, symbol_table: &SymbolTable) -> LIRContract {
    let Contract {
        requires,
        ensures,
        invariant,
    } = contract;

    LIRContract {
        requires: requires
            .into_iter()
            .map(|term| term_lir_from_hir(term, symbol_table))
            .collect(),
        ensures: ensures
            .into_iter()
            .map(|term| term_lir_from_hir(term, symbol_table))
            .collect(),
        invariant: invariant
            .into_iter()
            .map(|term| term_lir_from_hir(term, symbol_table))
            .collect(),
    }
}

mod term {
    use crate::{
        hir::contract::{Term, TermKind},
        lir::contract::Term as LIRTerm,
        symbol_table::SymbolTable,
    };

    /// Transform HIR term into LIR term.
    pub fn lir_from_hir(term: Term, symbol_table: &SymbolTable) -> LIRTerm {
        match term.kind {
            TermKind::Constant { constant } => LIRTerm::Constant { constant },
            TermKind::Identifier { id } => LIRTerm::Identifier {
                name: symbol_table.get_name(&id).clone(),
                scope: symbol_table.get_scope(&id).clone(),
            },
            TermKind::Binary { op, left, right } => LIRTerm::Binary {
                op,
                left: Box::new(lir_from_hir(*left, symbol_table)),
                right: Box::new(lir_from_hir(*right, symbol_table)),
            },
        }
    }
}
