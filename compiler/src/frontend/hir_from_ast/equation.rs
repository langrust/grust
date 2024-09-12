prelude! {
    ast::{
        equation::{ Arm, Equation, EventArmWhen, Instantiation, Match, MatchWhen },
        stmt::LetDecl,
    },
    hir::{ pattern, stream },
    itertools::Itertools,
}

use super::{HIRFromAST, SimpleCtxt};

impl<'a> HIRFromAST<SimpleCtxt<'a>> for Equation {
    type HIR = hir::Stmt<stream::Expr>;

    /// Pre-condition: equation's signal is already stored in symbol table.
    ///
    /// Post-condition: construct HIR equation and check identifiers good use.
    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
        let location = Location::default();

        // get signals defined by the equation
        let mut defined_signals = HashMap::new();
        self.get_signals(&mut defined_signals, ctxt.syms, ctxt.errors)?;

        match self {
            Equation::LocalDef(LetDecl {
                expression,
                typed_pattern: pattern,
                ..
            })
            | Equation::OutputDef(Instantiation {
                expression,
                pattern,
                ..
            }) => {
                let expression =
                    expression.hir_from_ast(&mut ctxt.add_pat_loc(Some(&pattern), &location))?;
                let pattern = pattern.hir_from_ast(&mut ctxt.add_loc(&location))?;
                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
            Equation::Match(Match {
                expression, arms, ..
            }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().hir_from_ast(&mut ctxt.add_loc(&location)))
                        .collect::<TRes<Vec<_>>>()?;
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        hir::stmt::init(hir::stmt::Kind::tuple(elements))
                    }
                };

                // for each arm, construct pattern guard and statements
                let arms = arms
                    .into_iter()
                    .map(
                        |Arm {
                             pattern,
                             guard,
                             equations,
                             ..
                         }| {
                            // transform pattern guard and equations into HIR
                            let (signals, pattern, guard, statements) = {
                                ctxt.syms.local();

                                // set local context: pattern signals + equations' signals
                                pattern.store(true, ctxt.syms, ctxt.errors)?;
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctxt.syms,
                                            ctxt.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform pattern guard and equations into HIR with local context
                                let pattern = pattern.hir_from_ast(&mut ctxt.add_loc(&location))?;
                                let guard = guard
                                    .map(|(_, expression)| {
                                        expression
                                            .hir_from_ast(&mut ctxt.add_pat_loc(None, &location))
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| equation.hir_from_ast(ctxt))
                                    .collect::<TRes<Vec<_>>>()?;

                                ctxt.syms.global();

                                (signals, pattern, guard, statements)
                            };

                            // create the tuple expression
                            let expression = {
                                // check defined signals are all the same
                                if defined_signals.len() != signals.len() {
                                    let error = Error::IncompatibleMatchStatements {
                                        expected: defined_signals.len(),
                                        received: signals.len(),
                                        location: location.clone(),
                                    };
                                    ctxt.errors.push(error);
                                    return Err(TerminationError);
                                }
                                let mut elements = defined_signals
                                    .keys()
                                    .map(|signal_name| {
                                        if let Some(id) = signals.get(signal_name) {
                                            Ok(stream::expr(stream::Kind::expr(
                                                hir::expr::Kind::ident(*id),
                                            )))
                                        } else {
                                            let error = Error::MissingMatchStatement {
                                                identifier: signal_name.clone(),
                                                location: location.clone(),
                                            };
                                            ctxt.errors.push(error);
                                            return Err(TerminationError);
                                        }
                                    })
                                    .collect::<TRes<Vec<_>>>()?;

                                // create the tuple expression
                                if elements.len() == 1 {
                                    elements.pop().unwrap()
                                } else {
                                    stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(
                                        elements,
                                    )))
                                }
                            };

                            Ok((pattern, guard, statements, expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                // construct the match expression
                let expression = stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                    expression.hir_from_ast(&mut ctxt.add_pat_loc(None, &location))?,
                    arms,
                )));

                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
            Equation::MatchWhen(MatchWhen { arms, .. }) => {
                // create the receiving pattern for the equation
                let defined_pattern = {
                    let mut elements = defined_signals.into_values().collect::<Vec<_>>();
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        ast::stmt::Pattern::tuple(ast::stmt::Tuple::new(elements))
                    }
                };

                let (
                    // map from event_id to index in tuple pattern
                    events_indices,
                    // default tuple pattern
                    default_pattern,
                ) = {
                    // create map from event_id to index in tuple pattern
                    let mut events_indices = HashMap::with_capacity(arms.len());
                    let mut idx = 0;
                    arms.iter()
                        .map(|event_arm| {
                            event_arm.pattern.place_events(
                                &mut events_indices,
                                &mut idx,
                                ctxt.syms,
                                ctxt.errors,
                            )
                        })
                        .collect::<TRes<()>>()?;

                    // default event_pattern tuple
                    let default_pattern: Vec<_> =
                        std::iter::repeat(pattern::init(pattern::Kind::default()))
                            .take(idx)
                            .collect();

                    (events_indices, default_pattern)
                };

                // default arm
                let default = {
                    // create tuple pattern
                    let elements = default_pattern.clone();
                    let pattern = pattern::init(pattern::Kind::tuple(elements));
                    // transform guard and equations into HIR with local context
                    let guard = None;
                    // create the tuple expression
                    let expression = defined_pattern.into_default_expr(
                        &HashMap::new(),
                        ctxt.syms,
                        ctxt.errors,
                    )?;

                    (pattern, guard, vec![], expression)
                };

                // for each arm construct hir pattern, guard and statements
                let mut match_arms = arms
                    .into_iter()
                    .map(
                        |EventArmWhen {
                             pattern: event_pattern,
                             guard,
                             equations,
                             ..
                         }| {
                            // transform event_pattern guard and equations into HIR
                            ctxt.syms.local();

                            // set local context + create matched pattern
                            let (matched_pattern, guard) = {
                                let mut elements = default_pattern.clone();
                                let opt_rising_edges = event_pattern.create_tuple_pattern(
                                    &mut elements,
                                    &events_indices,
                                    ctxt.syms,
                                    ctxt.errors,
                                )?;
                                let matched = pattern::init(pattern::Kind::tuple(elements));

                                // transform AST guard into HIR
                                let mut guard = guard
                                    .map(|(_, expression)| {
                                        expression
                                            .hir_from_ast(&mut ctxt.add_pat_loc(None, &location))
                                    })
                                    .transpose()?;
                                // add rising edge detection to the guard
                                if let Some(rising_edges) = opt_rising_edges {
                                    if let Some(old_guard) = guard.take() {
                                        guard = Some(hir::stream::expr(hir::stream::Kind::expr(
                                            hir::expr::Kind::binop(
                                                operator::BinaryOperator::And,
                                                old_guard,
                                                rising_edges,
                                            ),
                                        )));
                                    } else {
                                        guard = Some(rising_edges)
                                    }
                                };

                                (matched, guard)
                            };

                            // set and get local context: equations' signals
                            let signals = {
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctxt.syms,
                                            ctxt.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;
                                signals
                            };

                            // transform equations into HIR with local context
                            let statements = equations
                                .into_iter()
                                .map(|equation| equation.hir_from_ast(ctxt))
                                .collect::<TRes<Vec<_>>>()?;

                            ctxt.syms.global();

                            // create the tuple expression
                            let expression = defined_pattern.into_default_expr(
                                &signals,
                                ctxt.syms,
                                ctxt.errors,
                            )?;

                            Ok((matched_pattern, guard, statements, expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;
                match_arms.push(default);

                // construct the match expression
                let match_expr = {
                    // create tuple expression to match
                    let tuple_expr = {
                        let elements = events_indices
                            .iter()
                            .sorted_by_key(|(_, idx)| **idx)
                            .map(|(event_id, _)| {
                                stream::expr(stream::Kind::expr(hir::expr::Kind::ident(*event_id)))
                            })
                            .collect::<Vec<_>>();
                        stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)))
                    };
                    stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                        tuple_expr, match_arms,
                    )))
                };

                let pattern = defined_pattern.hir_from_ast(&mut ctxt.add_loc(&location))?;

                Ok(hir::Stmt {
                    pattern,
                    expression: match_expr,
                    location,
                })
            }
        }
    }
}
