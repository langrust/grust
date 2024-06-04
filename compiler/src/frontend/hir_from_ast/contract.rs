prelude! {
    ast::contract::{ClauseKind, Contract},
}

use super::HIRFromAST;

impl HIRFromAST for Contract {
    type HIR = hir::Contract;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let (requires, ensures, invariant) = self.clauses.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut requires, mut ensures, mut invariant), clause| {
                match clause.kind {
                    ClauseKind::Requires(_) => {
                        requires.push(clause.term.hir_from_ast(symbol_table, errors))
                    }
                    ClauseKind::Ensures(_) => {
                        ensures.push(clause.term.hir_from_ast(symbol_table, errors))
                    }
                    ClauseKind::Invariant(_) => {
                        invariant.push(clause.term.hir_from_ast(symbol_table, errors))
                    }
                    ClauseKind::Assert(_) => todo!(),
                };
                (requires, ensures, invariant)
            },
        );

        Ok(hir::Contract {
            requires: requires.into_iter().collect::<TRes<Vec<_>>>()?,
            ensures: ensures.into_iter().collect::<TRes<Vec<_>>>()?,
            invariant: invariant.into_iter().collect::<TRes<Vec<_>>>()?,
        })
    }
}

mod term {
    prelude! {
        ast::contract::{Binary, Implication, Term, Unary},
        operator::BinaryOperator,
    }

    use ast::contract::{Enumeration, EventImplication, ForAll, TimeoutImplication};

    use super::HIRFromAST;

    impl HIRFromAST for Term {
        type HIR = hir::contract::Term;

        fn hir_from_ast(
            self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Self::HIR> {
            let location = Location::default();
            match self {
                Term::Implication(Implication { left, right, .. }) => {
                    let left = left.hir_from_ast(symbol_table, errors)?;
                    let right = right.hir_from_ast(symbol_table, errors)?;

                    Ok(hir::contract::Term::new(
                        hir::contract::term::Kind::implication(left, right),
                        location,
                    ))
                }
                Term::Enumeration(Enumeration {
                    enum_name,
                    elem_name,
                }) => {
                    let enum_id =
                        symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
                    let element_id = symbol_table.get_enum_elem_id(
                        &elem_name,
                        &enum_name,
                        false,
                        location.clone(),
                        errors,
                    )?;
                    // TODO check elem is in enum
                    Ok(hir::contract::Term::new(
                        hir::contract::term::Kind::enumeration(enum_id, element_id),
                        location,
                    ))
                }
                Term::Unary(Unary { op, term }) => Ok(hir::contract::Term::new(
                    hir::contract::term::Kind::unary(op, term.hir_from_ast(symbol_table, errors)?),
                    location,
                )),
                Term::Binary(Binary { op, left, right }) => Ok(hir::contract::Term::new(
                    hir::contract::term::Kind::binary(
                        op,
                        left.hir_from_ast(symbol_table, errors)?,
                        right.hir_from_ast(symbol_table, errors)?,
                    ),
                    location,
                )),
                Term::Constant(constant) => Ok(hir::contract::Term::new(
                    hir::contract::term::Kind::constant(constant),
                    location,
                )),
                Term::Identifier(ident) => {
                    let id = symbol_table.get_identifier_id(
                        &ident,
                        false,
                        Location::default(),
                        errors,
                    )?;
                    Ok(hir::contract::Term::new(
                        hir::contract::term::Kind::ident(id),
                        location,
                    ))
                }
                Term::ForAll(ForAll {
                    ident, ty, term, ..
                }) => {
                    let ty = ty.hir_from_ast(&location, symbol_table, errors)?;
                    symbol_table.local();
                    let id = symbol_table.insert_identifier(
                        ident.clone(),
                        Some(ty),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    let term = term.hir_from_ast(symbol_table, errors)?;
                    symbol_table.global();
                    Ok(hir::contract::Term::new(
                        hir::contract::term::Kind::forall(id, term),
                        location,
                    ))
                }
                Term::EventImplication(EventImplication {
                    pattern,
                    event,
                    term,
                    ..
                }) => {
                    // get the event enumeration identifier
                    let event_enum_id =
                        symbol_table.get_event_enumeration_id(false, location.clone(), errors)?;
                    // get the event element identifier
                    let event_element_id: usize = symbol_table.get_event_element_id(
                        &event,
                        false,
                        location.clone(),
                        errors,
                    )?;
                    symbol_table.local();
                    // set pattern signal in local context
                    let pattern_id = symbol_table.insert_identifier(
                        pattern.clone(),
                        None,
                        true,
                        location.clone(),
                        errors,
                    )?;
                    // transform term into HIR
                    let right = term.hir_from_ast(symbol_table, errors)?;
                    symbol_table.global();
                    // construct right side of implication: `Event::E(pat) == event`
                    let event_id = symbol_table.get_event_id(false, location.clone(), errors)?;
                    let left = hir::contract::Term::new(
                        hir::contract::term::Kind::binary(
                            BinaryOperator::Eq,
                            hir::contract::Term::new(
                                hir::contract::term::Kind::event(
                                    event_enum_id,
                                    event_element_id,
                                    pattern_id,
                                ),
                                location.clone(),
                            ),
                            hir::contract::Term::new(
                                hir::contract::term::Kind::ident(event_id),
                                location.clone(),
                            ),
                        ),
                        location.clone(),
                    );
                    // construct result term: `when pat = e? => t` becomes `forall pat.idents, EventE(pat) == event => t`
                    let term = hir::contract::Term {
                        kind: hir::contract::term::Kind::forall(
                            pattern_id,
                            hir::contract::Term::new(
                                hir::contract::term::Kind::implication(left, right),
                                location.clone(),
                            ),
                        ),
                        location,
                    };
                    Ok(term)
                }
                Term::TimeoutImplication(TimeoutImplication { event, term, .. }) => {
                    // get the event enumeration identifier
                    let event_enum_id =
                        symbol_table.get_event_enumeration_id(false, location.clone(), errors)?;
                    // get the event element identifier
                    let event_element_id: usize = symbol_table.get_event_element_id(
                        &event,
                        false,
                        location.clone(),
                        errors,
                    )?;
                    // transform term into HIR
                    let right = term.hir_from_ast(symbol_table, errors)?;
                    // construct right side of implication: `Event::ETimeout == event`
                    let event_id = symbol_table.get_event_id(false, location.clone(), errors)?;
                    let left = hir::contract::Term::new(
                        hir::contract::term::Kind::binary(
                            BinaryOperator::Eq,
                            hir::contract::Term::new(
                                hir::contract::term::Kind::timeout(event_enum_id, event_element_id),
                                location.clone(),
                            ),
                            hir::contract::Term::new(
                                hir::contract::term::Kind::ident(event_id),
                                location.clone(),
                            ),
                        ),
                        location.clone(),
                    );
                    // construct result term: `timeout e? => t` becomes `Event::ETimeout == event => t`
                    let term = hir::contract::Term::new(
                        hir::contract::term::Kind::implication(left, right),
                        location,
                    );
                    Ok(term)
                }
            }
        }
    }
}
