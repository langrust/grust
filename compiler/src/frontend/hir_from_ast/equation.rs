use crate::ast::equation::{Arm, ArmWhen, Equation, Instanciation, Match, MatchWhen};
use crate::ast::pattern::{Pattern as ASTPattern, Tuple};
use crate::ast::statement::LetDeclaration;
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::dependencies::Dependencies;
use crate::hir::expression::ExpressionKind;
use crate::hir::pattern::{Pattern, PatternKind};
use crate::hir::stream_expression::StreamExpressionKind;
use crate::hir::{
    statement::Statement as HIRStatement,
    stream_expression::StreamExpression as HIRStreamExpression,
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Equation {
    type HIR = HIRStatement<HIRStreamExpression>;

    // precondition: equation's signal is already stored in symbol table
    // postcondition: construct HIR equation and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let location = Location::default();

        let declared_pattern = self.get_pattern();
        let pattern = declared_pattern.hir_from_ast(symbol_table, errors)?;

        match self {
            Equation::LocalDef(LetDeclaration { expression, .. })
            | Equation::OutputDef(Instanciation { expression, .. }) => Ok(HIRStatement {
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
                                .collect::<Vec<Result<_, _>>>()
                                .into_iter()
                                .collect::<Result<(), _>>()?;

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
                                .collect::<Vec<Result<_, _>>>()
                                .into_iter()
                                .collect::<Result<Vec<_>, _>>()?;

                            symbol_table.global();

                            Ok((pattern, guard, defined_signals, statements))
                        },
                    )
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

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

                        let elements = reference
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
                                    Ok(HIRStreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Identifier { id: *id },
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
                            .collect::<Vec<Result<_, _>>>()
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>()?;

                        let expression = HIRStreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Tuple { elements },
                            },
                            typing: None,
                            location: location.clone(),
                            dependencies: Dependencies::new(),
                        };
                        Ok((pattern, guard, statements, expression))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                // construct the match expression
                let expression = HIRStreamExpression {
                    kind: StreamExpressionKind::Expression {
                        expression: ExpressionKind::Match {
                            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                            arms,
                        },
                    },
                    typing: None,
                    location: location.clone(),
                    dependencies: Dependencies::new(),
                };

                Ok(HIRStatement {
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
                    .map(
                        |ArmWhen {
                             pattern,
                             event,
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
                                .collect::<Vec<Result<_, _>>>()
                                .into_iter()
                                .collect::<Result<(), _>>()?;

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
                                kind: PatternKind::Event {
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
                                .collect::<Vec<Result<_, _>>>()
                                .into_iter()
                                .collect::<Result<Vec<_>, _>>()?;

                            symbol_table.global();

                            Ok((pattern, guard, defined_signals, statements))
                        },
                    )
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

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

                        let elements = reference
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
                                    Ok(HIRStreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Identifier { id: *id },
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
                            .collect::<Vec<Result<_, _>>>()
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>()?;

                        let expression = HIRStreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Tuple { elements },
                            },
                            typing: None,
                            location: location.clone(),
                            dependencies: Dependencies::new(),
                        };
                        Ok((pattern, guard, statements, expression))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                // expression to match is the event enumeration
                let event_id = symbol_table.get_event_id(false, location.clone(), errors)?;
                let event_enum_expression = HIRStreamExpression {
                    kind: StreamExpressionKind::Event { event_id },
                    typing: None,
                    location: location.clone(),
                    dependencies: Dependencies::new(),
                };

                // construct the match expression
                let expression = HIRStreamExpression {
                    kind: StreamExpressionKind::Expression {
                        expression: ExpressionKind::Match {
                            expression: Box::new(event_enum_expression),
                            arms,
                        },
                    },
                    typing: None,
                    location: location.clone(),
                    dependencies: Dependencies::new(),
                };

                Ok(HIRStatement {
                    pattern,
                    expression,
                    location,
                })
            }
        }
    }
}

impl Equation {
    pub fn get_pattern(&self) -> ASTPattern {
        match self {
            Equation::LocalDef(declaration) => declaration.typed_pattern.clone(),
            Equation::OutputDef(instanciation) => instanciation.pattern.clone(),
            Equation::Match(Match { arms, .. }) => {
                let Arm { equations, .. } = arms.first().unwrap();
                let mut elements = equations
                    .iter()
                    .flat_map(|equation| equation.get_pattern().get_simple_patterns())
                    .collect::<Vec<_>>();
                if elements.len() == 1 {
                    elements.pop().unwrap()
                } else {
                    ASTPattern::Tuple(Tuple { elements })
                }
            }
            Equation::MatchWhen(MatchWhen { arms, .. }) => {
                let ArmWhen { equations, .. } = arms.first().unwrap();
                let mut elements = equations
                    .iter()
                    .flat_map(|equation| equation.get_pattern().get_simple_patterns())
                    .collect::<Vec<_>>();
                if elements.len() == 1 {
                    elements.pop().unwrap()
                } else {
                    ASTPattern::Tuple(Tuple { elements })
                }
            }
        }
    }

    pub fn store_local_declarations(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Option<Result<Vec<(String, usize)>, TerminationError>> {
        match self {
            Equation::LocalDef(declaration) => {
                Some(declaration.typed_pattern.store(true, symbol_table, errors))
            }
            Equation::OutputDef(_) => None,
            Equation::Match(Match { arms, .. }) => arms.first().map(|Arm { equations, .. }| {
                let local_declarations = equations
                    .iter()
                    .filter_map(|equation| equation.store_local_declarations(symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                Ok(local_declarations)
            }),
            Equation::MatchWhen(MatchWhen { arms, .. }) => {
                arms.first().map(|ArmWhen { equations, .. }| {
                    let local_declarations = equations
                        .iter()
                        .filter_map(|equation| {
                            equation.store_local_declarations(symbol_table, errors)
                        })
                        .collect::<Vec<Result<_, _>>>()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                    Ok(local_declarations)
                })
            }
        }
    }

    pub fn store_signals(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, TerminationError> {
        match self {
            Equation::LocalDef(declaration) => {
                declaration.typed_pattern.store(true, symbol_table, errors)
            }
            Equation::OutputDef(instanciation) => {
                instanciation.pattern.store(false, symbol_table, errors)
            }
            Equation::Match(Match { arms, .. }) => {
                arms.first().map_or(Ok(vec![]), |Arm { equations, .. }| {
                    Ok(equations
                        .iter()
                        .map(|equation| equation.store_signals(symbol_table, errors))
                        .collect::<Vec<Result<_, _>>>()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .flatten()
                        .collect())
                })
            }
            Equation::MatchWhen(MatchWhen { arms, .. }) => {
                arms.first()
                    .map_or(Ok(vec![]), |ArmWhen { equations, .. }| {
                        Ok(equations
                            .iter()
                            .map(|equation| equation.store_signals(symbol_table, errors))
                            .collect::<Vec<Result<_, _>>>()
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .flatten()
                            .collect())
                    })
            }
        }
    }
}
