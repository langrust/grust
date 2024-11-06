prelude! {
    hir::contract::Contract,
    lir::contract::Contract as LIRContract,
}

impl IntoLir<&'_ SymbolTable> for Contract {
    type Lir = LIRContract;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        let Contract {
            requires,
            ensures,
            invariant,
        } = self;

        LIRContract {
            requires: requires
                .into_iter()
                .map(|term| term.into_lir(symbol_table))
                .collect(),
            ensures: ensures
                .into_iter()
                .map(|term| term.into_lir(symbol_table))
                .collect(),
            invariant: invariant
                .into_iter()
                .map(|term| term.into_lir(symbol_table))
                .collect(),
        }
    }
}

mod term {
    prelude! {
        hir::contract::{Term, Kind},
    }

    impl IntoLir<&'_ SymbolTable> for Term {
        type Lir = lir::contract::Term;

        fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
            match self.kind {
                Kind::Constant { constant } => lir::contract::Term::literal(constant),
                Kind::Identifier { id } => {
                    let name = symbol_table.get_name(id);
                    match symbol_table.get_scope(id) {
                        Scope::Input => lir::contract::Term::input(name),
                        Scope::Output => lir::contract::Term::ident("result"), // todo: this will broke for components with multiple outputs
                        Scope::Local => lir::contract::Term::ident(name),
                        Scope::VeryLocal => unreachable!("you should not do that with this ident"),
                    }
                }
                Kind::Enumeration {
                    enum_id,
                    element_id,
                } => lir::contract::Term::enumeration(
                    symbol_table.get_name(enum_id).clone(),
                    symbol_table.get_name(element_id).clone(),
                    None,
                ),
                Kind::Unary { op, term } => {
                    lir::contract::Term::unop(op, term.into_lir(symbol_table))
                }
                Kind::Binary { op, left, right } => lir::contract::Term::binop(
                    op,
                    left.into_lir(symbol_table),
                    right.into_lir(symbol_table),
                ),
                Kind::ForAll { id, term } => {
                    let name = symbol_table.get_name(id);
                    let ty = symbol_table.get_type(id).clone();
                    let term = term.into_lir(symbol_table);
                    lir::contract::Term::forall(name, ty, term)
                }
                Kind::Implication { left, right } => lir::contract::Term::implication(
                    left.into_lir(symbol_table),
                    right.into_lir(symbol_table),
                ),
                Kind::PresentEvent { event_id, pattern } => match symbol_table.get_type(event_id) {
                    Typ::SMEvent { .. } => lir::contract::Term::some(lir::contract::Term::ident(
                        symbol_table.get_name(pattern),
                    )),
                    _ => unreachable!(),
                },
            }
        }
    }
}
