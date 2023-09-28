use crate::{
    ast::{expression::Expression, pattern::Pattern},
    common::r#type::Type,
    mir::expression::Expression as MIRExpression,
};

/// Transform HIR expression into MIR expression.
pub fn mir_from_hir(expression: Expression) -> MIRExpression {
    match expression {
        Expression::Constant { constant, .. } => MIRExpression::Literal { literal: constant },
        Expression::Call { id, .. } => MIRExpression::Identifier { identifier: id },
        Expression::Application {
            function_expression,
            inputs,
            ..
        } => MIRExpression::FunctionCall {
            function: Box::new(mir_from_hir(*function_expression)),
            arguments: inputs
                .into_iter()
                .map(|expression| mir_from_hir(expression))
                .collect(),
        },
        Expression::TypedAbstraction {
            inputs,
            expression,
            typing: Some(Type::Abstract(_, output_type)),
            ..
        } => MIRExpression::Lambda {
            inputs,
            output: *output_type,
            body: Box::new(mir_from_hir(*expression)),
        },
        Expression::Structure { name, fields, .. } => MIRExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(id, expression)| (id, mir_from_hir(expression)))
                .collect(),
        },
        Expression::Array { elements, .. } => MIRExpression::Array {
            elements: elements
                .into_iter()
                .map(|expression| mir_from_hir(expression))
                .collect(),
        },
        Expression::Match {
            expression, arms, ..
        } => MIRExpression::Match {
            matched: Box::new(mir_from_hir(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, expression)| {
                    (
                        pattern,
                        guard.map(|expression| mir_from_hir(expression)),
                        mir_from_hir(expression),
                    )
                })
                .collect(),
        },
        Expression::When {
            id,
            option,
            present,
            default,
            location,
            ..
        } => MIRExpression::Match {
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
        _ => unreachable!(),
    }
}
