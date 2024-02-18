use crate::ast::expression::ExpressionKind;
use crate::ast::stream_expression::{StreamExpression, StreamExpressionKind};
use crate::error::{Error, TerminationError};
use crate::hir::{
    dependencies::Dependencies,
    stream_expression::{
        StreamExpression as HIRStreamExpression, StreamExpressionKind as HIRStreamExpressionKind,
    },
};
use crate::symbol_table::{SymbolKind, SymbolTable};

use super::HIRFromAST;

impl HIRFromAST for StreamExpression {
    type HIR = HIRStreamExpression;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let StreamExpression { kind, location } = self;

        match kind {
            StreamExpressionKind::Expression { expression } => {
                // check if it is a node expression (ie: node_id(intputs).signal_id)
                match &expression {
                    ExpressionKind::FieldAccess {
                        expression,
                        field: signal,
                    } => {
                        match &expression.kind {
                            StreamExpressionKind::Expression {
                                expression:
                                    ExpressionKind::Application {
                                        function_expression,
                                        inputs: inputs_stream_expressions,
                                    },
                            } => match &function_expression.kind {
                                StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier { id: node },
                                } if symbol_table.is_node(&node, false) => {
                                    let node_id = symbol_table.get_node_id(
                                        &node,
                                        false,
                                        location.clone(),
                                        errors,
                                    )?;
                                    let node_symbol = symbol_table
                                        .get_symbol(&node_id)
                                        .expect("there should be a symbol")
                                        .clone();
                                    match node_symbol.kind() {
                                        SymbolKind::Node {
                                            is_component,
                                            inputs,
                                            outputs,
                                            ..
                                        } => {
                                            // if component raise error: component can not be called
                                            if *is_component {
                                                let error = Error::ComponentCall {
                                                    name: node.clone(),
                                                    location: location.clone(),
                                                };
                                                errors.push(error);
                                                return Err(TerminationError);
                                            }

                                            // check inputs and node_inputs have the same length
                                            if inputs.len() != inputs_stream_expressions.len() {
                                                let error = Error::IncompatibleInputsNumber {
                                                    given_inputs_number: inputs_stream_expressions
                                                        .len(),
                                                    expected_inputs_number: inputs.len(),
                                                    location: location.clone(),
                                                };
                                                errors.push(error);
                                                return Err(TerminationError);
                                            }

                                            let output_id = *outputs
                                                .get(signal)
                                                .expect("this is not an output"); // TODO: make it an error to raise
                                            return Ok(HIRStreamExpression {
                                                kind: HIRStreamExpressionKind::NodeApplication {
                                                    node_id,
                                                    output_id,
                                                    inputs: inputs_stream_expressions
                                                        .into_iter()
                                                        .zip(inputs)
                                                        .map(|(input, id)| {
                                                            Ok((
                                                                *id,
                                                                input.clone().hir_from_ast(
                                                                    symbol_table,
                                                                    errors,
                                                                )?,
                                                            ))
                                                        })
                                                        .collect::<Vec<Result<_, _>>>()
                                                        .into_iter()
                                                        .collect::<Result<Vec<_>, _>>()?,
                                                },
                                                typing: None,
                                                location: location.clone(),
                                                dependencies: Dependencies::new(),
                                            });
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => Ok(()),
                            },
                            _ => Ok(()),
                        }
                    }
                    _ => Ok(()),
                }?;
                Ok(HIRStreamExpression {
                    kind: HIRStreamExpressionKind::Expression {
                        expression: expression.hir_from_ast(&location, symbol_table, errors)?,
                    },
                    typing: None,
                    location: location,
                    dependencies: Dependencies::new(),
                })
            }
            StreamExpressionKind::FollowedBy {
                constant,
                expression,
            } => Ok(HIRStreamExpression {
                kind: HIRStreamExpressionKind::FollowedBy {
                    constant,
                    expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                },
                typing: None,
                location: location,
                dependencies: Dependencies::new(),
            }),
        }
    }
}
