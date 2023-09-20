use crate::lir::expression::{Arm, Expression as LIRExpression, FieldExpression};
use crate::lir::pattern::Pattern;
use crate::mir::expression::Expression;

use super::{
    block::lir_from_mir as block_lir_from_mir, pattern::lir_from_mir as pattern_lir_from_mir,
    r#type::lir_from_mir as type_lir_from_mir,
};

/// Transform MIR expression into LIR expression.
pub fn lir_from_mir(expression: Expression) -> LIRExpression {
    match expression {
        Expression::Literal { literal } => LIRExpression::Literal { literal },
        Expression::Identifier { identifier } => LIRExpression::Identifier { identifier },
        Expression::MemoryAccess { identifier } => {
            let self_call = LIRExpression::Identifier {
                identifier: String::from("self"),
            };
            LIRExpression::FieldAccess {
                expression: Box::new(self_call),
                field: identifier,
            }
        }
        Expression::Structure { name, fields } => {
            let fields = fields
                .into_iter()
                .map(|(name, expression)| FieldExpression {
                    name,
                    expression: lir_from_mir(expression),
                })
                .collect();
            LIRExpression::Structure { name, fields }
        }
        Expression::Array { elements } => todo!(),
        Expression::Block { block } => LIRExpression::Block {
            block: block_lir_from_mir(block),
        },
        Expression::FunctionCall {
            function,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|expression| lir_from_mir(expression))
                .collect();
            LIRExpression::FunctionCall {
                function: Box::new(lir_from_mir(*function)),
                arguments,
            }
        }
        Expression::NodeCall {
            node_identifier,
            input_name,
            input_fields,
        } => {
            let self_call = LIRExpression::Identifier {
                identifier: String::from("self"),
            };
            let receiver = LIRExpression::FieldAccess {
                expression: Box::new(self_call),
                field: node_identifier,
            };
            let input_fields = input_fields
                .into_iter()
                .map(|(name, expression)| FieldExpression {
                    name,
                    expression: lir_from_mir(expression),
                })
                .collect();
            let argument = LIRExpression::Structure {
                name: input_name,
                fields: input_fields,
            };
            LIRExpression::MethodCall {
                receiver: Box::new(receiver),
                method: String::from("step"),
                arguments: vec![argument],
            }
        }
        Expression::FieldAccess { expression, field } => LIRExpression::FieldAccess {
            expression: Box::new(lir_from_mir(*expression)),
            field,
        },
        Expression::Lambda {
            inputs,
            output,
            body,
        } => {
            let inputs = inputs
                .into_iter()
                .map(|(identifier, r#type)| Pattern::Typed {
                    pattern: Box::new(Pattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier,
                    }),
                    r#type: type_lir_from_mir(r#type),
                })
                .collect();
            LIRExpression::Closure {
                r#move: false,
                inputs,
                output: Some(type_lir_from_mir(output)),
                body: Box::new(lir_from_mir(*body)),
            }
        }
        Expression::IfThenElse {
            condition,
            then_branch,
            else_branch,
        } => LIRExpression::IfThenElse {
            condition: Box::new(lir_from_mir(*condition)),
            then_branch: block_lir_from_mir(then_branch),
            else_branch: Some(block_lir_from_mir(else_branch)),
        },
        Expression::Match { matched, arms } => {
            let arms = arms
                .into_iter()
                .map(|(pattern, guard, body)| Arm {
                    pattern: pattern_lir_from_mir(pattern),
                    guard: guard.map(|expression| lir_from_mir(expression)),
                    body: lir_from_mir(body),
                })
                .collect();
            LIRExpression::Match {
                matched: Box::new(lir_from_mir(*matched)),
                arms,
            }
        }
    }
}
