prelude! {
    ast::contract::{ClauseKind, Contract},
}

impl IntoHir<hir::ctx::Simple<'_>> for Contract {
    type Hir = hir::Contract;

    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        let (requires, ensures, invariant) = self.clauses.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut requires, mut ensures, mut invariant), clause| {
                match clause.kind {
                    ClauseKind::Requires(_) => requires.push(clause.term.into_hir(ctxt)),
                    ClauseKind::Ensures(_) => ensures.push(clause.term.into_hir(ctxt)),
                    ClauseKind::Invariant(_) => invariant.push(clause.term.into_hir(ctxt)),
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
    }

    impl<'a> IntoHir<hir::ctx::Simple<'a>> for Term {
        type Hir = hir::contract::Term;

        fn into_hir(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
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
                    let left = left.into_hir(ctxt)?;
                    let right = right.into_hir(ctxt)?;

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
                    hir::contract::Kind::unary(op, term.into_hir(ctxt)?),
                    None,
                    location,
                )),
                Term::Binary(Binary { op, left, right }) => Ok(hir::contract::Term::new(
                    hir::contract::Kind::binary(op, left.into_hir(ctxt)?, right.into_hir(ctxt)?),
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
                    let ty = ty.into_hir(&mut ctxt.add_loc(&location))?;
                    ctxt.syms.local();
                    let id = ctxt.syms.insert_identifier(
                        ident.clone(),
                        Some(ty),
                        true,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    let term = term.into_hir(ctxt)?;
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
                    let right = term.into_hir(ctxt)?;
                    ctxt.syms.global();
                    // construct right side of implication: `PresentEvent(pat) == event`
                    let left = hir::contract::Term::new(
                        hir::contract::Kind::binary(
                            BOp::Eq,
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
