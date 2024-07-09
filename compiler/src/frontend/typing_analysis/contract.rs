prelude! {
    frontend::TypeAnalysis,
    hir::{Contract, contract},
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
                name: symbol_table.get_name(*enum_id).clone(),
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
                ty.eq_check(&Typ::Boolean, self.location.clone(), errors)?;
                Typ::Boolean
            }
            contract::term::Kind::Implication { left, right } => {
                left.typing(symbol_table, errors)?;
                let ty = left.typing.as_ref().unwrap();
                ty.eq_check(&Typ::Boolean, self.location.clone(), errors)?;
                right.typing(symbol_table, errors)?;
                let ty = right.typing.as_ref().unwrap();
                ty.eq_check(&Typ::Boolean, self.location.clone(), errors)?;
                Typ::Boolean
            }
            contract::term::Kind::PresentEvent { event_id, pattern } => {
                let typing = symbol_table.get_type(*event_id).clone();
                match &typing {
                    Typ::SMEvent(expected_type) | Typ::SMTimeout(expected_type) => {
                        symbol_table.set_type(*pattern, *expected_type.clone());
                    }
                    _ => unreachable!(),
                };
                typing
            }
            contract::term::Kind::TimeoutEvent { event_id } => {
                let typing = symbol_table.get_type(*event_id).clone();
                match &typing {
                    Typ::SMTimeout(_) => (),
                    _ => panic!("error, should be 'event timeout'"),
                };
                typing
            }
        };
        self.typing = Some(ty);
        Ok(())
    }
}
