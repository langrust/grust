prelude! {
    hir::contract::Contract,
    lir::{contract::Contract as LIRContract, item::import::Import},
}

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
            imports.push(Import::creusot("ensures"));
            imports.push(Import::creusot("requires"));
        } else {
            if !self.ensures.is_empty() {
                imports.push(Import::creusot("ensures"));
            }
            if !self.requires.is_empty() {
                imports.push(Import::creusot("requires"));
            }
        }

        imports
    }
}

mod term {
    prelude! {
        hir::contract::{Term, term},
    }

    impl super::LIRFromHIR for Term {
        type LIR = lir::contract::Term;

        fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
            match self.kind {
                term::Kind::Constant { constant } => lir::contract::Term::literal(constant),
                term::Kind::Identifier { id } => {
                    let name = symbol_table.get_name(id);
                    match symbol_table.get_scope(id) {
                        Scope::Input => lir::contract::Term::input(name),
                        Scope::Memory => lir::contract::Term::mem(name),
                        Scope::Output => lir::contract::Term::ident("result"),
                        Scope::Local => lir::contract::Term::ident(name),
                    }
                }
                term::Kind::Enumeration {
                    enum_id,
                    element_id,
                } => lir::contract::Term::enumeration(
                    symbol_table.get_name(enum_id).clone(),
                    symbol_table.get_name(element_id).clone(),
                    None,
                ),
                term::Kind::Unary { op, term } => {
                    lir::contract::Term::unop(op, term.lir_from_hir(symbol_table))
                }
                term::Kind::Binary { op, left, right } => lir::contract::Term::binop(
                    op,
                    left.lir_from_hir(symbol_table),
                    right.lir_from_hir(symbol_table),
                ),
                term::Kind::ForAll { id, term } => {
                    let name = symbol_table.get_name(id);
                    let ty = symbol_table.get_type(id).clone();
                    let term = term.lir_from_hir(symbol_table);
                    lir::contract::Term::forall(name, ty, term)
                }
                term::Kind::Implication { left, right } => lir::contract::Term::implication(
                    left.lir_from_hir(symbol_table),
                    right.lir_from_hir(symbol_table),
                ),
                term::Kind::Event {
                    event_enum_id,
                    event_element_id,
                    pattern,
                } => match symbol_table.get_type(event_element_id) {
                    Typ::SMEvent(_) => lir::contract::Term::enumeration(
                        symbol_table.get_name(event_enum_id).clone(),
                        symbol_table.get_name(event_element_id).clone(),
                        Some(lir::contract::Term::ident(symbol_table.get_name(pattern))),
                    ),
                    Typ::SMTimeout(_) => lir::contract::Term::enumeration(
                        symbol_table.get_name(event_enum_id).clone(),
                        symbol_table.get_name(event_element_id).clone(),
                        Some(lir::contract::Term::ok(lir::contract::Term::ident(
                            symbol_table.get_name(pattern),
                        ))),
                    ),
                    _ => unreachable!(),
                },
                term::Kind::TimeoutEvent {
                    event_enum_id,
                    event_element_id,
                } => lir::contract::Term::enumeration(
                    symbol_table.get_name(event_enum_id).clone(),
                    symbol_table.get_name(event_element_id).clone(),
                    Some(lir::contract::Term::err()),
                ),
            }
        }
    }
}
