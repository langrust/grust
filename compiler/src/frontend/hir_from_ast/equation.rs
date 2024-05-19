use crate::ast::equation::{Arm, Equation, Instanciation, Match};
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
        match self {
            Equation::LocalDef(LetDeclaration {
                typed_pattern: pattern,
                expression,
                ..
            })
            | Equation::OutputDef(Instanciation {
                pattern,
                expression,
                ..
            }) => {
                let mut pattern = pattern.hir_from_ast(symbol_table, errors)?;
                pattern.construct_statement_type(symbol_table, errors)?;
                debug_assert!(pattern.typing.is_some());

                Ok(HIRStatement {
                    pattern,
                    expression: expression.hir_from_ast(symbol_table, errors)?,
                    location,
                })
            }
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
                            pattern.store(symbol_table, errors)?;
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

                // check signals are all the same
                let reference = new_arms.first().unwrap().2.clone();
                new_arms
                    .iter()
                    .map(|(_, _, defined_signals, _)| {
                        if reference.len() != defined_signals.len() {
                            todo!("create error");
                            return Err(TerminationError);
                        }

                        reference
                            .iter()
                            .map(|(signal_name, signal_id)| {
                                let signal_type = symbol_table.get_type(*signal_id);
                                let signal_scope = symbol_table.get_scope(*signal_id);
                                let test: bool = defined_signals
                                    .iter()
                                    .position(|(other_name, other_id)| {
                                        let other_type = symbol_table.get_type(*other_id);
                                        let other_scope = symbol_table.get_scope(*other_id);
                                        signal_name == other_name
                                            && signal_type == other_type
                                            && signal_scope == other_scope
                                    })
                                    .is_none();
                                if test {
                                    todo!("create error");
                                    return Err(TerminationError);
                                } else {
                                    Ok(())
                                }
                            })
                            .collect::<Vec<Result<_, _>>>()
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>()?;

                        Ok(())
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                // create the tuple pattern
                let pattern = Pattern {
                    kind: PatternKind::Tuple {
                        elements: reference
                            .iter()
                            .map(|(_, id)| Pattern {
                                kind: PatternKind::Identifier { id: *id },
                                typing: None,
                                location: location.clone(),
                            })
                            .collect(),
                    },
                    typing: None,
                    location: location.clone(),
                };

                // for every arm, create the tuple expression
                let arms = new_arms
                    .into_iter()
                    .map(|(pattern, guard, _, statements)| {
                        let expression = HIRStreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Tuple {
                                    elements: reference
                                        .iter()
                                        .map(|(_, id)| HIRStreamExpression {
                                            kind: StreamExpressionKind::Expression {
                                                expression: ExpressionKind::Identifier { id: *id },
                                            },
                                            typing: None,
                                            location: location.clone(),
                                            dependencies: Dependencies::new(),
                                        })
                                        .collect(),
                                },
                            },
                            typing: None,
                            location: location.clone(),
                            dependencies: Dependencies::new(),
                        };
                        (pattern, guard, statements, expression)
                    })
                    .collect();

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
        }
    }
}

impl Equation {
    pub fn store_local_declarations(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Option<Result<Vec<(String, usize)>, TerminationError>> {
        match self {
            Equation::LocalDef(declaration) => {
                Some(declaration.typed_pattern.store(symbol_table, errors))
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
        }
    }

    pub fn store_signals(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, TerminationError> {
        match self {
            Equation::LocalDef(declaration) => {
                declaration.typed_pattern.store(symbol_table, errors)
            }
            Equation::OutputDef(instanciation) => instanciation.pattern.store(symbol_table, errors),
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
        }
    }
}
