prelude! {
    ast::{
        equation::{
            Arm, ArmWhen, DefaultArmWhen, Equation, EventArmWhen,
            Instantiation, Match, MatchWhen, EventPattern
        },
        stmt::LetDecl,
    },
    hir::{ pattern, stream },
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

        let declared_pattern = self.get_pattern();
        let pattern = declared_pattern.hir_from_ast(symbol_table, errors)?;

        match self {
            Equation::LocalDef(LetDecl { expression, .. })
            | Equation::OutputDef(Instantiation { expression, .. }) => Ok(hir::Stmt {
                pattern,
                expression: expression.hir_from_ast(symbol_table, errors)?,
                location,
            }),
            Equation::Match(Match {
                expression, arms, ..
            }) => {
                // for each arm construct hir pattern, guard and statements
                let new_arms = arms
                    .into_iter()
                    .map(
                        |Arm {
                             pattern,
                             guard,
                             equations,
                             ..
                         }| {
                            symbol_table.local();

                            // set local context: pattern signals + equations' signals
                            pattern.store(true, symbol_table, errors)?;
                            let mut defined_signals = vec![];
                            equations
                                .iter()
                                .map(|equation| {
                                    // store equations' signals in the local context
                                    let mut equation_signals =
                                        equation.store_signals(symbol_table, errors)?;
                                    defined_signals.append(&mut equation_signals);
                                    Ok(())
                                })
                                .collect::<TRes<()>>()?;

                            // transform into HIR
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

                            Ok((pattern, guard, defined_signals, statements))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                // for every arm, check signals are all the same
                // and create the tuple expression
                let reference = new_arms.first().unwrap().2.clone();
                let arms = new_arms
                    .into_iter()
                    .map(|(pattern, guard, defined_signals, statements)| {
                        if reference.len() != defined_signals.len() {
                            let error = Error::IncompatibleMatchStatements {
                                expected: reference.len(),
                                received: defined_signals.len(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            return Err(TerminationError);
                        }

                        let mut elements = reference
                            .iter()
                            .map(|(signal_name, signal_id)| {
                                let signal_scope = symbol_table.get_scope(*signal_id);
                                let position =
                                    defined_signals.iter().position(|(other_name, other_id)| {
                                        let other_scope = symbol_table.get_scope(*other_id);
                                        signal_name == other_name && signal_scope == other_scope
                                    });
                                if let Some(index) = position {
                                    let (_, id) = defined_signals.get(index).unwrap();
                                    Ok(stream::expr(stream::Kind::expr(hir::expr::Kind::ident(
                                        *id,
                                    ))))
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

                        let expression = if elements.len() == 1 {
                            elements.pop().unwrap()
                        } else {
                            stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)))
                        };
                        Ok((pattern, guard, statements, expression))
                    })
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
            Equation::MatchWhen(MatchWhen { arms, .. }) => {
                // create map from event_id to index in tuple pattern
                let mut events_indices = HashMap::with_capacity(arms.len());
                let mut idx = 0;
                arms.iter()
                    .map(|arm| match arm {
                        ArmWhen::EventArmWhen(EventArmWhen { pattern, .. }) => pattern
                            .place_events(&mut events_indices, &mut idx, symbol_table, errors),
                        ArmWhen::Default(_) => Ok(()),
                    })
                    .collect::<TRes<_>>()?;

                // default tuple
                let tuple: Vec<_> = std::iter::repeat(pattern::init(pattern::Kind::default()))
                    .take(idx)
                    .collect();

                // for each arm construct hir pattern, guard and statements
                let new_arms = arms
                    .into_iter()
                    .map(|arm| match arm {
                        ArmWhen::EventArmWhen(EventArmWhen {
                            pattern,
                            guard,
                            equations,
                            ..
                        }) => {
                            symbol_table.local();

                            // set local context: pattern signals + equations' signals
                            let defined_signals =
                                local_context(&pattern, &equations, symbol_table, errors)?;
                            // create tuple pattern
                            let mut elements = tuple.clone();
                            pattern.create_tuple_pattern(
                                &mut elements,
                                &events_indices,
                                symbol_table,
                                errors,
                            )?;
                            let pattern = pattern::init(pattern::Kind::tuple(elements));
                            // transform guard and equations
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

                            Ok((pattern, guard, defined_signals, statements))
                        }
                        ArmWhen::Default(DefaultArmWhen { equations, .. }) => {
                            symbol_table.local();

                            // set local context: pattern signals + equations' signals
                            let mut defined_signals = vec![];
                            equations
                                .iter()
                                .map(|equation| {
                                    // store equations' signals in the local context
                                    let mut equation_signals =
                                        equation.store_signals(symbol_table, errors)?;
                                    defined_signals.append(&mut equation_signals);
                                    Ok(())
                                })
                                .collect::<TRes<()>>()?;

                            // create tuple pattern
                            let pattern = pattern::init(pattern::Kind::tuple(tuple.clone()));
                            // // defensive option
                            // let mut elements = tuple.clone();
                            // map.iter().for_each(|(event_id, idx)| {
                            //     let no_event_pattern =
                            //         pattern::init(pattern::Kind::absent(*event_id));
                            //     *elements.get_mut(*idx).unwrap() = no_event_pattern.clone();
                            // });

                            let guard = None;
                            let statements = equations
                                .into_iter()
                                .map(|equation| equation.hir_from_ast(symbol_table, errors))
                                .collect::<TRes<Vec<_>>>()?;

                            symbol_table.global();

                            Ok((pattern, guard, defined_signals, statements))
                        }
                    })
                    .collect::<TRes<Vec<_>>>()?;

                // for every arm, check signals are all the same
                // and create the tuple expression
                let reference = new_arms.first().unwrap().2.clone();
                let arms = new_arms
                    .into_iter()
                    .map(|(pattern, guard, defined_signals, statements)| {
                        if reference.len() != defined_signals.len() {
                            let error = Error::IncompatibleMatchStatements {
                                expected: reference.len(),
                                received: defined_signals.len(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            return Err(TerminationError);
                        }

                        let mut elements = reference
                            .iter()
                            .map(|(signal_name, signal_id)| {
                                let signal_scope = symbol_table.get_scope(*signal_id);
                                let position =
                                    defined_signals.iter().position(|(other_name, other_id)| {
                                        let other_scope = symbol_table.get_scope(*other_id);
                                        signal_name == other_name && signal_scope == other_scope
                                    });
                                if let Some(index) = position {
                                    let (_, id) = defined_signals.get(index).unwrap();
                                    Ok(stream::expr(stream::Kind::expr(hir::expr::Kind::ident(
                                        *id,
                                    ))))
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

                        let expression = if elements.len() == 1 {
                            elements.pop().unwrap()
                        } else {
                            stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)))
                        };
                        Ok((pattern, guard, statements, expression))
                    })
                    .collect::<TRes<Vec<_>>>()?;

                // create tuple expression to match
                let mut elements =
                    std::iter::repeat(stream::expr(stream::Kind::expr(hir::expr::Kind::ident(0))))
                        .take(idx)
                        .collect::<Vec<_>>();
                events_indices.iter().for_each(|(event_id, idx)| {
                    let event_expr =
                        stream::expr(stream::Kind::expr(hir::expr::Kind::ident(*event_id)));
                    *elements.get_mut(*idx).unwrap() = event_expr;
                });
                let expr = stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)));

                // construct the match expression
                let match_expr =
                    stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(expr, arms)));

                Ok(hir::Stmt {
                    pattern,
                    expression: match_expr,
                    location,
                })
            }
        }
    }
}

fn local_context(
    pattern: &EventPattern,
    equations: &Vec<Equation>,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> TRes<Vec<(String, usize)>> {
    // set local context: pattern signals + equations' signals
    pattern.store(symbol_table, errors)?;
    let mut defined_signals = vec![];
    equations
        .iter()
        .map(|equation| {
            // store equations' signals in the local context
            let mut equation_signals = equation.store_signals(symbol_table, errors)?;
            defined_signals.append(&mut equation_signals);
            Ok(())
        })
        .collect::<TRes<()>>()?;
    // return locally defined signals
    Ok(defined_signals)
}
