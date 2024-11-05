prelude! {
    frontend::TypeAnalysis,
    hir::{Contract, contract},
    macro2::Span,
}

impl TypeAnalysis for Contract {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let Contract {
            requires,
            ensures,
            invariant,
        } = self;

        for term in requires.iter_mut() {
            term.typing(symbol_table, errors)?
        }

        for term in ensures.iter_mut() {
            term.typing(symbol_table, errors)?
        }

        for term in invariant.iter_mut() {
            term.typing(symbol_table, errors)?
        }

        Ok(())
    }
}

impl TypeAnalysis for contract::Term {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let ty = match &mut self.kind {
            contract::term::Kind::Constant { constant } => constant.get_type(),
            contract::term::Kind::Identifier { id } => symbol_table.get_type(*id).clone(),
            contract::term::Kind::Enumeration { enum_id, .. } => Typ::Enumeration {
                name: Ident::new(symbol_table.get_name(*enum_id), Span::call_site()),
                id: *enum_id,
            },
            contract::term::Kind::Unary { op, term } => {
                term.typing(symbol_table, errors)?;
                let ty = term.typing.as_ref().unwrap().clone();
                let mut unop_type = op.get_type();
                unop_type.apply(vec![ty], self.location.clone(), errors)?
            }
            contract::term::Kind::Binary { op, left, right } => {
                left.typing(symbol_table, errors)?;
                let left_type = left.typing.as_ref().unwrap().clone();
                right.typing(symbol_table, errors)?;
                let right_type = right.typing.as_ref().unwrap().clone();
                let mut binop_type = op.get_type();
                binop_type.apply(vec![left_type, right_type], self.location.clone(), errors)?
            }
            contract::term::Kind::ForAll { term, .. } => {
                term.typing(symbol_table, errors)?;
                let ty = term.typing.as_ref().unwrap();
                ty.eq_check(&Typ::bool(), self.location.clone(), errors)?;
                Typ::bool()
            }
            contract::term::Kind::Implication { left, right } => {
                left.typing(symbol_table, errors)?;
                let ty = left.typing.as_ref().unwrap();
                ty.eq_check(&Typ::bool(), self.location.clone(), errors)?;
                right.typing(symbol_table, errors)?;
                let ty = right.typing.as_ref().unwrap();
                ty.eq_check(&Typ::bool(), self.location.clone(), errors)?;
                ty.clone()
            }
            contract::term::Kind::PresentEvent { event_id, pattern } => {
                let typing = symbol_table.get_type(*event_id).clone();
                match &typing {
                    Typ::SMEvent { ty, .. } => {
                        symbol_table.set_type(*pattern, *ty.clone());
                    }
                    _ => unreachable!(),
                };
                typing
            }
        };
        self.typing = Some(ty);
        Ok(())
    }
}
