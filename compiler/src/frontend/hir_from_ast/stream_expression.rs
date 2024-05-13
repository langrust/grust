use crate::ast::expression::{
    Application, Array, Binop, FieldAccess, IfThenElse, Structure, Tuple, Unop,
};
use crate::ast::stream_expression::{FollowedBy, StreamExpression};
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::expression::ExpressionKind;
use crate::hir::{
    dependencies::Dependencies,
    stream_expression::{StreamExpression as HIRStreamExpression, StreamExpressionKind},
};
use crate::symbol_table::{SymbolKind, SymbolTable};

use super::HIRFromAST;

impl HIRFromAST for StreamExpression {
    type HIR = HIRStreamExpression;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR stream expression and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let location = Location::default();
        let kind = match self {
            StreamExpression::FieldAccess(FieldAccess { expression, field }) => {
                // check if it is a node expression (ie: node_id(intputs).signal_id)
                match *expression {
                    StreamExpression::Application(Application {
                        function_expression,
                        inputs: inputs_stream_expressions,
                    }) => match *function_expression {
                        StreamExpression::Identifier(node)
                            if symbol_table.is_node(&node, false) =>
                        {
                            let node_id =
                                symbol_table.get_node_id(&node, false, location.clone(), errors)?;
                            let node_symbol = symbol_table
                                .get_symbol(node_id)
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
                                            given_inputs_number: inputs_stream_expressions.len(),
                                            expected_inputs_number: inputs.len(),
                                            location: location.clone(),
                                        };
                                        errors.push(error);
                                        return Err(TerminationError);
                                    }

                                    let output_id = *outputs.get(&field).ok_or_else(|| {
                                        let error = Error::UnknownOuputSignal {
                                            node_name: node.clone(),
                                            signal_name: field,
                                            location: location.clone(),
                                        };
                                        errors.push(error);
                                        TerminationError
                                    })?;

                                    StreamExpressionKind::NodeApplication {
                                        node_id,
                                        output_id,
                                        inputs: inputs_stream_expressions
                                            .into_iter()
                                            .zip(inputs)
                                            .map(|(input, id)| {
                                                Ok((
                                                    *id,
                                                    input
                                                        .clone()
                                                        .hir_from_ast(symbol_table, errors)?,
                                                ))
                                            })
                                            .collect::<Vec<Result<_, _>>>()
                                            .into_iter()
                                            .collect::<Result<Vec<_>, _>>()?,
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                        function_expression => StreamExpressionKind::Expression {
                            expression: ExpressionKind::FieldAccess {
                                expression: Box::new(HIRStreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Application {
                                            function_expression: Box::new(
                                                function_expression
                                                    .hir_from_ast(symbol_table, errors)?,
                                            ),
                                            inputs: inputs_stream_expressions
                                                .into_iter()
                                                .map(|input| {
                                                    input.clone().hir_from_ast(symbol_table, errors)
                                                })
                                                .collect::<Vec<Result<_, _>>>()
                                                .into_iter()
                                                .collect::<Result<Vec<_>, _>>()?,
                                        },
                                    },
                                    typing: None,
                                    location: location.clone(),
                                    dependencies: Dependencies::new(),
                                }),
                                field,
                            },
                        },
                    },
                    expression => StreamExpressionKind::Expression {
                        expression: ExpressionKind::FieldAccess {
                            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                            field,
                        },
                    },
                }
            }
            StreamExpression::FollowedBy(FollowedBy {
                constant,
                expression,
            }) => {
                // check the constant expression is indeed constant
                constant.check_is_constant(symbol_table, errors)?;

                StreamExpressionKind::FollowedBy {
                    constant: Box::new(constant.hir_from_ast(symbol_table, errors)?),
                    expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                }
            }
            StreamExpression::Constant(constant) => StreamExpressionKind::Expression {
                expression: ExpressionKind::Constant { constant },
            },
            StreamExpression::Identifier(id) => {
                let id = symbol_table
                    .get_identifier_id(&id, false, location.clone(), &mut vec![])
                    .or_else(|_| {
                        symbol_table.get_function_id(&id, false, location.clone(), errors)
                    })?;
                StreamExpressionKind::Expression {
                    expression: ExpressionKind::Identifier { id },
                }
            }
            StreamExpression::Unop(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Binop(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::IfThenElse(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Application(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::TypedAbstraction(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(&location, symbol_table, errors)?,
            },
            StreamExpression::Structure(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(&location, symbol_table, errors)?,
            },
            StreamExpression::Tuple(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Enumeration(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast::<StreamExpression>(
                    &location,
                    symbol_table,
                    errors,
                )?,
            },
            StreamExpression::Array(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Match(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::TupleElementAccess(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Map(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Fold(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Sort(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
            StreamExpression::Zip(expression) => StreamExpressionKind::Expression {
                expression: expression.hir_from_ast(symbol_table, errors)?,
            },
        };
        Ok(HIRStreamExpression {
            kind,
            typing: None,
            location,
            dependencies: Dependencies::new(),
        })
    }
}

impl StreamExpression {
    fn check_is_constant(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match &self {
            // Constant by default
            StreamExpression::Constant { .. } | StreamExpression::Enumeration { .. } => Ok(()),
            // Not constant by default
            StreamExpression::TypedAbstraction { .. }
            | StreamExpression::Match { .. }
            | StreamExpression::FieldAccess { .. }
            | StreamExpression::TupleElementAccess { .. }
            | StreamExpression::Map { .. }
            | StreamExpression::Fold { .. }
            | StreamExpression::Sort { .. }
            | StreamExpression::Zip { .. }
            | StreamExpression::FollowedBy { .. } => {
                let error = Error::ExpectConstant {
                    location: Location::default(),
                };
                errors.push(error);
                Err(TerminationError)
            }
            // It depends
            StreamExpression::Identifier(id) => {
                // check id exists
                let id = symbol_table
                    .get_identifier_id(&id, false, Location::default(), &mut vec![])
                    .or_else(|_| {
                        symbol_table.get_function_id(&id, false, Location::default(), errors)
                    })?;
                // check it is a function or and operator
                if symbol_table.is_function(id) {
                    Ok(())
                } else {
                    let error = Error::ExpectConstant {
                        location: Location::default(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
            }
            StreamExpression::Unop(Unop { expression, .. }) => {
                expression.check_is_constant(symbol_table, errors)
            }
            StreamExpression::Binop(Binop {
                left_expression,
                right_expression,
                ..
            }) => {
                left_expression.check_is_constant(symbol_table, errors)?;
                right_expression.check_is_constant(symbol_table, errors)
            }
            StreamExpression::IfThenElse(IfThenElse {
                expression,
                true_expression,
                false_expression,
                ..
            }) => {
                expression.check_is_constant(symbol_table, errors)?;
                true_expression.check_is_constant(symbol_table, errors)?;
                false_expression.check_is_constant(symbol_table, errors)
            }
            StreamExpression::Application(Application {
                function_expression,
                inputs,
            }) => {
                function_expression.check_is_constant(symbol_table, errors)?;
                inputs
                    .iter()
                    .map(|expression| expression.check_is_constant(symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()
            }
            StreamExpression::Structure(Structure { fields, .. }) => fields
                .iter()
                .map(|(_, expression)| expression.check_is_constant(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<_, _>>(),
            StreamExpression::Array(Array { elements })
            | StreamExpression::Tuple(Tuple { elements }) => elements
                .iter()
                .map(|expression| expression.check_is_constant(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<_, _>>(),
        }
    }
}
