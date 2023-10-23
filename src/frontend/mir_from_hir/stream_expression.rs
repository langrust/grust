use crate::{
    ast::pattern::Pattern,
    hir::stream_expression::StreamExpression,
    mir::{block::Block, expression::Expression, statement::Statement},
};

use super::{
    equation::mir_from_hir as equation_mir_from_hir,
    expression::mir_from_hir as expression_mir_from_hir,
};

/// Transform HIR stream expression into MIR expression.
pub fn mir_from_hir(stream_expression: StreamExpression) -> Expression {
    match stream_expression {
        StreamExpression::Constant { constant, .. } => Expression::Literal { literal: constant },
        StreamExpression::SignalCall { id, .. } => Expression::Identifier { identifier: id },
        StreamExpression::MapApplication {
            function_expression,
            inputs,
            ..
        } => Expression::FunctionCall {
            function: Box::new(expression_mir_from_hir(function_expression)),
            arguments: inputs
                .into_iter()
                .map(|expression| mir_from_hir(expression))
                .collect(),
        },
        StreamExpression::Structure { name, fields, .. } => Expression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(id, expression)| (id, mir_from_hir(expression)))
                .collect(),
        },
        StreamExpression::Array { elements, .. } => Expression::Array {
            elements: elements
                .into_iter()
                .map(|expression| mir_from_hir(expression))
                .collect(),
        },
        StreamExpression::Match {
            expression, arms, ..
        } => Expression::Match {
            matched: Box::new(mir_from_hir(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, body, expression)| {
                    (
                        pattern,
                        guard.map(|expression| mir_from_hir(expression)),
                        if body.is_empty() {
                            mir_from_hir(expression)
                        } else {
                            let mut statements = body
                                .into_iter()
                                .map(|equation| equation_mir_from_hir(equation))
                                .collect::<Vec<_>>();
                            statements.push(Statement::ExpressionLast {
                                expression: mir_from_hir(expression),
                            });
                            Expression::Block {
                                block: Block { statements },
                            }
                        },
                    )
                })
                .collect(),
        },
        StreamExpression::When {
            id,
            option,
            present,
            default,
            location,
            ..
        } => Expression::Match {
            matched: Box::new(mir_from_hir(*option)),
            arms: vec![
                (
                    Pattern::Some {
                        pattern: Box::new(Pattern::Identifier {
                            name: id,
                            location: location.clone(),
                        }),
                        location: location.clone(),
                    },
                    None,
                    mir_from_hir(*present),
                ),
                (Pattern::None { location }, None, mir_from_hir(*default)),
            ],
        },
        StreamExpression::UnitaryNodeApplication {
            node,
            signal,
            inputs,
            ..
        } => Expression::NodeCall {
            node_identifier: node.clone() + &signal,
            input_name: node.clone() + &signal + "Inputs",
            input_fields: inputs
                .into_iter()
                .map(|(id, expression)| (id, mir_from_hir(expression)))
                .collect(),
        },
        _ => unreachable!(),
    }
}
