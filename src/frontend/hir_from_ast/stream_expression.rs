use crate::ast::stream_expression::{StreamExpression, StreamExpressionKind};
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::expression::hir_from_ast as expression_hir_from_ast;
use crate::hir::{
    dependencies::Dependencies,
    stream_expression::{
        StreamExpression as HIRStreamExpression, StreamExpressionKind as HIRStreamExpressionKind,
    },
};
use crate::symbol_table::{SymbolKind, SymbolTable};

/// Transform AST stream expressions into HIR stream expressions.
pub fn hir_from_ast(
    stream_expression: StreamExpression,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRStreamExpression, TerminationError> {
    let StreamExpression {
        kind,
        location,
    } = stream_expression;

    match kind {
        StreamExpressionKind::Expression { expression } => Ok(HIRStreamExpression {
            kind: HIRStreamExpressionKind::Expression {
                expression: expression_hir_from_ast(expression, symbol_table, errors)?,
            },
            typing: None,
            location: location,
            dependencies: Dependencies::new(),
        }),
        StreamExpressionKind::FollowedBy {
            constant,
            expression,
        } => Ok(HIRStreamExpression {
            kind: HIRStreamExpressionKind::FollowedBy {
                constant,
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            },
            typing: None,
            location: location,
            dependencies: Dependencies::new(),
        }),
        StreamExpressionKind::NodeApplication {
            node,
            inputs: inputs_stream_expressions,
            signal,
        } => {
            let node_id = symbol_table.get_node_id(&node, false, location.clone(), errors)?;
            let node_symbol = symbol_table
                .get_symbol(&node_id)
                .expect("there should be a symbol");
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
                            given_inputs_number: inputs_stream_expressions.len(),
                            expected_inputs_number: inputs.len(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        return Err(TerminationError);
                    }

                    let output_id = *outputs.get(&signal).expect("this is not an output"); // TODO: make it an error to raise
                    Ok(HIRStreamExpression {
                        kind: HIRStreamExpressionKind::NodeApplication {
                            node_id,
                            output_id,
                            inputs: inputs_stream_expressions
                                .into_iter()
                                .zip(inputs)
                                .map(|(input, id)| {
                                    Ok((*id, hir_from_ast(input, symbol_table, errors)?))
                                })
                                .collect::<Vec<Result<_, _>>>()
                                .into_iter()
                                .collect::<Result<Vec<_>, _>>()?,
                        },
                        typing: None,
                        location: location,
                        dependencies: Dependencies::new(),
                    })
                }
                _ => unreachable!(),
            }
        }
    }
}
