use crate::{
    hir::contract::Contract,
    lir::{contract::Contract as LIRContract, item::import::Import},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Contract {
    type LIR = LIRContract;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let Contract {
            requires,
            ensures,
            invariant,
        } = self;

        LIRContract {
            requires: requires
                .into_iter()
                .map(|term| term.lir_from_hir(symbol_table))
                .collect(),
            ensures: ensures
                .into_iter()
                .map(|term| term.lir_from_hir(symbol_table))
                .collect(),
            invariant: invariant
                .into_iter()
                .map(|term| term.lir_from_hir(symbol_table))
                .collect(),
        }
    }

    fn get_imports(&self, _symbol_table: &SymbolTable) -> Vec<Import> {
        let mut imports = vec![];

        if !self.invariant.is_empty() {
            imports.push(Import::Creusot(String::from("ensures")));
            imports.push(Import::Creusot(String::from("requires")))
        } else {
            if !self.ensures.is_empty() {
                imports.push(Import::Creusot(String::from("ensures")))
            }
            if !self.requires.is_empty() {
                imports.push(Import::Creusot(String::from("requires")))
            }
        }

        imports
    }
}

mod term {
    use crate::{
        hir::contract::{Term, TermKind},
        lir::contract::Term as LIRTerm,
        symbol_table::SymbolTable,
    };

    use super::LIRFromHIR;

    impl LIRFromHIR for Term {
        type LIR = LIRTerm;

        fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
            match self.kind {
                TermKind::Constant { constant } => LIRTerm::Constant { constant },
                TermKind::Identifier { id } => LIRTerm::Identifier {
                    name: symbol_table.get_name(id).clone(),
                    scope: symbol_table.get_scope(id).clone(),
                },
                TermKind::Unary { op, term } => LIRTerm::Unary {
                    op,
                    term: Box::new(term.lir_from_hir(symbol_table)),
                },
                TermKind::Binary { op, left, right } => LIRTerm::Binary {
                    op,
                    left: Box::new(left.lir_from_hir(symbol_table)),
                    right: Box::new(right.lir_from_hir(symbol_table)),
                },
            }
        }
    }
}
