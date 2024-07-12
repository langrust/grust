prelude! {
    hir::contract::Contract,
    lir::contract::Contract as LIRContract,
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
                        Scope::Output => lir::contract::Term::ident("result"), // todo: this will broke for components with multiple outputs
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
                term::Kind::PresentEvent { event_id, pattern } => {
                    match symbol_table.get_type(event_id) {
                        Typ::SMEvent { .. } => lir::contract::Term::some(
                            lir::contract::Term::ident(symbol_table.get_name(pattern)),
                        ),
                        Typ::SMTimeout { .. } => {
                            lir::contract::Term::some(lir::contract::Term::ok(
                                lir::contract::Term::ident(symbol_table.get_name(pattern)),
                            ))
                        }
                        _ => unreachable!(),
                    }
                }
                term::Kind::TimeoutEvent { event_id } => match symbol_table.get_type(event_id) {
                    Typ::SMTimeout { .. } => lir::contract::Term::some(lir::contract::Term::err()),
                    _ => unreachable!(),
                },
            }
        }
    }
}
