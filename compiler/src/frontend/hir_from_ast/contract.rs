prelude! {
    ast::contract::{ClauseKind, Contract},
}

use super::{HIRFromAST, SimpleCtxt};

impl<'a> HIRFromAST<SimpleCtxt<'a>> for Contract {
    type HIR = hir::Contract;

    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
        let (requires, ensures, invariant) = self.clauses.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut requires, mut ensures, mut invariant), clause| {
                match clause.kind {
                    ClauseKind::Requires(_) => requires.push(clause.term.hir_from_ast(ctxt)),
                    ClauseKind::Ensures(_) => ensures.push(clause.term.hir_from_ast(ctxt)),
                    ClauseKind::Invariant(_) => invariant.push(clause.term.hir_from_ast(ctxt)),
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
        ast::contract::{Binary, Implication, Term, Unary, Enumeration, EventImplication, ForAll},
        operator::BinaryOperator,
    }

    use super::{HIRFromAST, SimpleCtxt};

    impl<'a> HIRFromAST<SimpleCtxt<'a>> for Term {
        type HIR = hir::contract::Term;

        fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
            let location = Location::default();
            match self {
                Term::Result(_) => {
                    let id =
                        ctxt.syms
                            .get_function_result_id(false, location.clone(), ctxt.errors)?;
                    Ok(hir::contract::Term::new(
                        hir::contract::Kind::ident(id),
                        None,
                        location,
                    ))
                }
                Term::Implication(Implication { left, right, .. }) => {
                    let left = left.hir_from_ast(ctxt)?;
                    let right = right.hir_from_ast(ctxt)?;

                    Ok(hir::contract::Term::new(
                        hir::contract::Kind::implication(left, right),
                        None,
                        location,
                    ))
                }
                Term::Enumeration(Enumeration {
                    enum_name,
                    elem_name,
                }) => {
                    let enum_id =
                        ctxt.syms
                            .get_enum_id(&enum_name, false, location.clone(), ctxt.errors)?;
                    let element_id = ctxt.syms.get_enum_elem_id(
                        &elem_name,
                        &enum_name,
                        false,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    // TODO check elem is in enum
                    Ok(hir::contract::Term::new(
                        hir::contract::Kind::enumeration(enum_id, element_id),
                        None,
                        location,
                    ))
                }
                Term::Unary(Unary { op, term }) => Ok(hir::contract::Term::new(
                    hir::contract::Kind::unary(op, term.hir_from_ast(ctxt)?),
                    None,
                    location,
                )),
                Term::Binary(Binary { op, left, right }) => Ok(hir::contract::Term::new(
                    hir::contract::Kind::binary(
                        op,
                        left.hir_from_ast(ctxt)?,
                        right.hir_from_ast(ctxt)?,
                    ),
                    None,
                    location,
                )),
                Term::Constant(constant) => Ok(hir::contract::Term::new(
                    hir::contract::Kind::constant(constant),
                    None,
                    location,
                )),
                Term::Identifier(ident) => {
                    let id = ctxt.syms.get_identifier_id(
                        &ident,
                        false,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    Ok(hir::contract::Term::new(
                        hir::contract::Kind::ident(id),
                        None,
                        location,
                    ))
                }
                Term::ForAll(ForAll {
                    ident, ty, term, ..
                }) => {
                    let ty = ty.hir_from_ast(&mut ctxt.add_loc(&location))?;
                    ctxt.syms.local();
                    let id = ctxt.syms.insert_identifier(
                        ident.clone(),
                        Some(ty),
                        true,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    let term = term.hir_from_ast(ctxt)?;
                    ctxt.syms.global();
                    Ok(hir::contract::Term::new(
                        hir::contract::Kind::forall(id, term),
                        None,
                        location,
                    ))
                }
                Term::EventImplication(EventImplication {
                    pattern,
                    event,
                    term,
                    ..
                }) => {
                    // get the event identifier
                    let event_id = ctxt.syms.get_identifier_id(
                        &event,
                        false,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    ctxt.syms.local();
                    // set pattern signal in local context
                    let pattern_id = ctxt.syms.insert_identifier(
                        pattern.clone(),
                        None,
                        true,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    // transform term into HIR
                    let right = term.hir_from_ast(ctxt)?;
                    ctxt.syms.global();
                    // construct right side of implication: `PresentEvent(pat) == event`
                    let left = hir::contract::Term::new(
                        hir::contract::Kind::binary(
                            BinaryOperator::Eq,
                            hir::contract::Term::new(
                                hir::contract::Kind::present(event_id, pattern_id),
                                None,
                                location.clone(),
                            ),
                            hir::contract::Term::new(
                                hir::contract::Kind::ident(event_id),
                                None,
                                location.clone(),
                            ),
                        ),
                        None,
                        location.clone(),
                    );
                    // construct result term: `when pat = e? => t` becomes `forall pat, PresentEvent(pat) == event => t`
                    let term = hir::contract::Term::new(
                        hir::contract::Kind::forall(
                            pattern_id,
                            hir::contract::Term::new(
                                hir::contract::Kind::implication(left, right),
                                None,
                                location.clone(),
                            ),
                        ),
                        None,
                        location,
                    );
                    Ok(term)
                }
            }
        }
    }
}
