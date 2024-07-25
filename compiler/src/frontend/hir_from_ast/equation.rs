prelude! {
    ast::{
        equation::{
            Arm, DefaultArmWhen, Equation, EventArmWhen,
            Instantiation, Match, MatchWhen,
        },
        stmt::LetDecl,
    },
    hir::{ pattern, stream },
    itertools::Itertools,
}

use super::HIRFromAST;

impl HIRFromAST for Equation {
    type HIR = hir::Stmt<stream::Expr>;

    /// Pre-condition: equation's signal is already stored in symbol table.
    ///
    /// Post-condition: construct HIR equation and check identifiers good use.
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();

        // get signals defined by the equation
        let mut defined_signals = HashMap::new();
        self.get_signals(&mut defined_signals, symbol_table, errors)?;

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
            }) => Ok(hir::Stmt {
                pattern: pattern.hir_from_ast(symbol_table, errors)?,
                expression: expression.hir_from_ast(symbol_table, errors)?,
                location,
            }),
            Equation::Match(Match {
                expression, arms, ..
            }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().hir_from_ast(symbol_table, errors))
                        .collect::<TRes<Vec<_>>>()?;
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        hir::pattern::init(hir::pattern::Kind::tuple(elements))
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
                                symbol_table.local();

                                // set local context: pattern signals + equations' signals
                                pattern.store(true, symbol_table, errors)?;
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            symbol_table,
                                            errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform pattern guard and equations into HIR with local context
                                let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                                let guard = guard
                                    .map(|(_, expression)| {
                                        expression.hir_from_ast(symbol_table, errors)
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| equation.hir_from_ast(symbol_table, errors))
                                    .collect::<TRes<Vec<_>>>()?;

                                symbol_table.global();

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
                                    errors.push(error);
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
                                            errors.push(error);
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
                    expression.hir_from_ast(symbol_table, errors)?,
                    arms,
                )));

                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
            Equation::MatchWhen(MatchWhen { arms, default, .. }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().hir_from_ast(symbol_table, errors))
                        .collect::<TRes<Vec<_>>>()?;
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        hir::pattern::init(hir::pattern::Kind::tuple(elements))
                    }
                };

                // create map from event_id to index in tuple pattern and default tuple pattern
                let (events_indices, events_nb, no_event_tuple) = {
                    // create map from event_id to index in tuple pattern
                    let mut events_indices = HashMap::with_capacity(arms.len());
                    let mut idx = 0;
                    arms.iter()
                        .map(|event_arm| {
                            event_arm.pattern.place_events(
                                &mut events_indices,
                                &mut idx,
                                symbol_table,
                                errors,
                            )
                        })
                        .collect::<TRes<_>>()?;

                    // default event_pattern tuple
                    let no_event_tuple: Vec<_> =
                        std::iter::repeat(pattern::init(pattern::Kind::default()))
                            .take(idx)
                            .collect();

                    (events_indices, idx, no_event_tuple)
                };

                // get the default arm if present
                let (always_defined, opt_default) = if let Some(DefaultArmWhen {
                    equations, ..
                }) = default
                {
                    let (signals, pattern, guard, statements) = {
                        // transform event_pattern guard and equations into HIR
                        symbol_table.local();

                        // set local context: no event + equations' signals
                        let mut signals = HashMap::new();
                        equations
                            .iter()
                            .map(|equation| {
                                // store equations' signals in the local context
                                equation.store_signals(true, &mut signals, symbol_table, errors)
                            })
                            .collect::<TRes<()>>()?;

                        // create tuple pattern
                        let elements = no_event_tuple.clone();
                        let pattern = pattern::init(pattern::Kind::tuple(elements));
                        // transform guard and equations into HIR with local context
                        let guard = None;
                        let statements = equations
                            .into_iter()
                            .map(|equation| equation.hir_from_ast(symbol_table, errors))
                            .collect::<TRes<Vec<_>>>()?;

                        symbol_table.global();

                        (signals, pattern, guard, statements)
                    };

                    // create the tuple expression
                    let expression = {
                        let mut elements = defined_signals
                            .keys()
                            .map(|signal_name| {
                                if let Some(id) = signals.get(signal_name) {
                                    stream::expr(stream::Kind::expr(hir::expr::Kind::ident(*id)))
                                } else {
                                    stream::expr(stream::Kind::none_event())
                                }
                            })
                            .collect::<Vec<_>>();
                        if elements.len() == 1 {
                            elements.pop().unwrap()
                        } else {
                            stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)))
                        }
                    };

                    (signals, Some((pattern, guard, statements, expression)))
                } else {
                    (Default::default(), None)
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
                            let (signals, pattern, guard, statements) = {
                                // transform event_pattern guard and equations into HIR
                                symbol_table.local();

                                // set local context: events + equations' signals
                                // create tuple pattern: it stores events identifiers
                                let mut elements = no_event_tuple.clone();
                                event_pattern.create_tuple_pattern(
                                    &mut elements,
                                    &events_indices,
                                    symbol_table,
                                    errors,
                                )?;
                                let pattern = pattern::init(pattern::Kind::tuple(elements));
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            symbol_table,
                                            errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform guard and equations into HIR with local context
                                let guard = guard
                                    .map(|(_, expression)| {
                                        expression.hir_from_ast(symbol_table, errors)
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| {
                                        let mut def_signals = HashMap::new();
                                        equation
                                            .get_signals(&mut def_signals, symbol_table, errors)
                                            .expect("internal bug");

                                        let mut stmt =
                                            equation.hir_from_ast(symbol_table, errors)?;

                                        if def_signals
                                            .keys()
                                            .any(|name| !always_defined.contains_key(name))
                                        {
                                            stmt.expression = stream::expr(
                                                stream::Kind::some_event(stmt.expression),
                                            );
                                        }

                                        Ok(stmt)
                                    })
                                    .collect::<TRes<Vec<_>>>()?;

                                symbol_table.global();

                                (signals, pattern, guard, statements)
                            };

                            // create the tuple expression
                            let expression = {
                                let mut elements = defined_signals
                                    .keys()
                                    .map(|signal_name| {
                                        if let Some(id) = signals.get(signal_name) {
                                            stream::expr(stream::Kind::expr(
                                                hir::expr::Kind::ident(*id),
                                            ))
                                        } else {
                                            stream::expr(stream::Kind::none_event())
                                        }
                                    })
                                    .collect::<Vec<_>>();
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
                opt_default.map(|default| match_arms.push(default));

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
                        debug_assert!(elements.len() == events_nb);
                        stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)))
                    };
                    stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                        tuple_expr, match_arms,
                    )))
                };

                Ok(hir::Stmt {
                    pattern,
                    expression: match_expr,
                    location,
                })
            }
        }
    }
}
