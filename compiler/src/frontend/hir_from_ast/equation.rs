prelude! {
    ast::{
        equation::{
            Arm, ArmWhen, DefaultArmWhen, Equation, EventArmWhen, Instantiation, Match, MatchWhen,
            TimeoutArmWhen,
        },
        stmt::LetDecl,
    },
    hir::{
        Dependencies, Pattern, pattern,
        stream,
    },
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
                                    Ok(stream::Expr {
                                        kind: stream::Kind::Expression {
                                            expression: hir::expr::Kind::Identifier { id: *id },
                                        },
                                        typing: None,
                                        location: location.clone(),
                                        dependencies: Dependencies::new(),
                                    })
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
                            stream::Expr {
                                kind: stream::Kind::Expression {
                                    expression: hir::expr::Kind::Tuple { elements },
                                },
                                typing: None,
                                location: location.clone(),
                                dependencies: Dependencies::new(),
                            }
                        };
                        Ok((pattern, guard, statements, expression))
                    })
                    .collect::<TRes<Vec<_>>>()?;

                // construct the match expression
                let expression = stream::Expr {
                    kind: stream::Kind::Expression {
                        expression: hir::expr::Kind::Match {
                            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                            arms,
                        },
                    },
                    typing: None,
                    location: location.clone(),
                    dependencies: Dependencies::new(),
                };

                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
            Equation::MatchWhen(MatchWhen { arms, .. }) => {
                // get the event enumeration identifier
                let event_enum_id: usize =
                    symbol_table.get_event_enumeration_id(false, location.clone(), errors)?;

                // for each arm construct hir pattern, guard and statements
                let new_arms = arms
                    .into_iter()
                    .map(|arm| match arm {
                        ArmWhen::EventArmWhen(EventArmWhen {
                            pattern,
                            event,
                            guard,
                            equations,
                            ..
                        }) => {
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

                            // get the event element identifier
                            let event_element_id: usize = symbol_table.get_event_element_id(
                                &event.to_string(),
                                false,
                                location.clone(),
                                errors,
                            )?;

                            // transform into HIR
                            let inner_pattern = pattern.hir_from_ast(symbol_table, errors)?;
                            let pattern = Pattern {
                                kind: pattern::Kind::Event {
                                    event_enum_id,
                                    event_element_id,
                                    pattern: Box::new(inner_pattern),
                                },
                                typing: None,
                                location: location.clone(),
                            };
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
                        ArmWhen::TimeoutArmWhen(TimeoutArmWhen {
                            event, equations, ..
                        }) => {
                            symbol_table.local();

                            // set local context: equations' signals
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

                            // get the event element identifier
                            let event_element_id: usize = symbol_table.get_event_element_id(
                                &event.to_string(),
                                false,
                                location.clone(),
                                errors,
                            )?;

                            // transform into HIR
                            let pattern = Pattern {
                                kind: pattern::Kind::TimeoutEvent {
                                    event_enum_id,
                                    event_element_id,
                                },
                                typing: None,
                                location: location.clone(),
                            };
                            let guard = None;
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

                            // transform into HIR
                            let pattern = Pattern {
                                kind: pattern::Kind::NoEvent { event_enum_id },
                                typing: None,
                                location: location.clone(),
                            };
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
                                    Ok(stream::Expr {
                                        kind: stream::Kind::Expression {
                                            expression: hir::expr::Kind::Identifier { id: *id },
                                        },
                                        typing: None,
                                        location: location.clone(),
                                        dependencies: Dependencies::new(),
                                    })
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
                            stream::Expr {
                                kind: stream::Kind::Expression {
                                    expression: hir::expr::Kind::Tuple { elements },
                                },
                                typing: None,
                                location: location.clone(),
                                dependencies: Dependencies::new(),
                            }
                        };
                        Ok((pattern, guard, statements, expression))
                    })
                    .collect::<TRes<Vec<_>>>()?;

                // expression to match is the event enumeration
                let event_id = symbol_table.get_event_id(false, location.clone(), errors)?;
                let event_enum_expression = stream::Expr {
                    kind: stream::Kind::Event { event_id },
                    typing: None,
                    location: location.clone(),
                    dependencies: Dependencies::new(),
                };

                // construct the match expression
                let expression = stream::Expr {
                    kind: stream::Kind::Expression {
                        expression: hir::expr::Kind::Match {
                            expression: Box::new(event_enum_expression),
                            arms,
                        },
                    },
                    typing: None,
                    location: location.clone(),
                    dependencies: Dependencies::new(),
                };

                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
        }
    }
}
