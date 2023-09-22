use crate::frontend::lir_from_mir::expression::lir_from_mir as expression_lir_from_mir;
use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::frontend::lir_from_mir::statement::lir_from_mir as statement_lir_from_mir;
use crate::lir::block::Block;
use crate::lir::expression::{Expression, FieldExpression};
use crate::lir::item::implementation::AssociatedItem;
use crate::lir::item::signature::{Receiver, Signature};
use crate::lir::r#type::Type as LIRType;
use crate::lir::statement::Statement;
use crate::mir::item::node_file::state::step::{StateElementStep, Step};

/// Transform MIR step into LIR implementation method.
pub fn lir_from_mir(step: Step) -> AssociatedItem {
    let signature = Signature {
        public_visibility: true,
        name: String::from("step"),
        receiver: Some(Receiver {
            reference: false,
            mutable: false,
        }),
        inputs: vec![(
            String::from("input"),
            LIRType::Identifier {
                identifier: step.node_name.clone() + "Input",
            },
        )],
        output: LIRType::Tuple {
            elements: vec![
                LIRType::Identifier {
                    identifier: step.node_name.clone() + "State",
                },
                type_lir_from_mir(step.output_type),
            ],
        },
    };
    let mut statements = step
        .body
        .into_iter()
        .map(|statement| statement_lir_from_mir(statement))
        .collect::<Vec<_>>();

    let fields = step
        .state_elements_step
        .into_iter()
        .map(
            |StateElementStep {
                 identifier,
                 expression,
             }| FieldExpression {
                name: identifier,
                expression: expression_lir_from_mir(expression),
            },
        )
        .collect();
    let statement = Statement::ExpressionLast(Expression::Tuple {
        elements: vec![
            Expression::Structure {
                name: step.node_name + "State",
                fields,
            },
            expression_lir_from_mir(step.output_expression),
        ],
    });

    statements.push(statement);

    let body = Block { statements };
    AssociatedItem::AssociatedMethod { signature, body }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::constant::Constant;
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::item::node_file::state::step::lir_from_mir;
    use crate::lir::block::Block;
    use crate::lir::expression::{Expression as LIRExpression, FieldExpression};
    use crate::lir::item::implementation::AssociatedItem;
    use crate::lir::item::signature::{Receiver, Signature};
    use crate::lir::pattern::Pattern;
    use crate::lir::r#type::Type as LIRType;
    use crate::lir::statement::r#let::Let;
    use crate::lir::statement::Statement as LIRStatement;
    use crate::mir::expression::Expression;
    use crate::mir::item::node_file::state::step::{StateElementStep, Step};
    use crate::mir::statement::Statement;

    #[test]
    fn should_create_lir_associated_method_from_mir_node_init() {
        let init = Step {
            node_name: format!("Node"),
            output_type: Type::Integer,
            body: vec![
                Statement::Let {
                    identifier: format!("o"),
                    expression: Expression::FieldAccess {
                        expression: Box::new(Expression::Identifier {
                            identifier: format!("self"),
                        }),
                        field: format!("mem_i"),
                    },
                },
                Statement::LetTuple {
                    identifiers: vec![format!("new_called_node_state"), format!("y")],
                    expression: Expression::NodeCall {
                        node_identifier: format!("called_node_state"),
                        input_name: format!("CalledNodeInput"),
                        input_fields: vec![],
                    },
                },
            ],
            state_elements_step: vec![
                StateElementStep {
                    identifier: format!("mem_i"),
                    expression: Expression::FunctionCall {
                        function: Box::new(Expression::Identifier {
                            identifier: format!(" + "),
                        }),
                        arguments: vec![
                            Expression::Identifier {
                                identifier: format!("o"),
                            },
                            Expression::Literal {
                                literal: Constant::Integer(1),
                            },
                        ],
                    },
                },
                StateElementStep {
                    identifier: format!("called_node_state"),
                    expression: Expression::Identifier {
                        identifier: format!("new_called_node_state"),
                    },
                },
            ],
            output_expression: Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    identifier: format!(" + "),
                }),
                arguments: vec![
                    Expression::Identifier {
                        identifier: format!("o"),
                    },
                    Expression::Identifier {
                        identifier: format!("y"),
                    },
                ],
            },
        };
        let control = AssociatedItem::AssociatedMethod {
            signature: Signature {
                public_visibility: true,
                name: format!("step"),
                receiver: Some(Receiver {
                    reference: false,
                    mutable: false,
                }),
                inputs: vec![(
                    format!("input"),
                    LIRType::Identifier {
                        identifier: format!("NodeInput"),
                    },
                )],
                output: LIRType::Tuple {
                    elements: vec![
                        LIRType::Identifier {
                            identifier: format!("NodeState"),
                        },
                        LIRType::Identifier {
                            identifier: format!("i64"),
                        },
                    ],
                },
            },
            body: Block {
                statements: vec![
                    LIRStatement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: false,
                            identifier: format!("o"),
                        },
                        expression: LIRExpression::FieldAccess {
                            expression: Box::new(LIRExpression::Identifier {
                                identifier: format!("self"),
                            }),
                            field: format!("mem_i"),
                        },
                    }),
                    LIRStatement::Let(Let {
                        pattern: Pattern::Tuple {
                            elements: vec![
                                Pattern::Identifier {
                                    reference: false,
                                    mutable: false,
                                    identifier: format!("new_called_node_state"),
                                },
                                Pattern::Identifier {
                                    reference: false,
                                    mutable: false,
                                    identifier: format!("y"),
                                },
                            ],
                        },
                        expression: LIRExpression::MethodCall {
                            receiver: Box::new(LIRExpression::FieldAccess {
                                expression: Box::new(LIRExpression::Identifier {
                                    identifier: format!("self"),
                                }),
                                field: format!("called_node_state"),
                            }),
                            method: format!("step"),
                            arguments: vec![LIRExpression::Structure {
                                name: format!("CalledNodeInput"),
                                fields: vec![],
                            }],
                        },
                    }),
                    LIRStatement::ExpressionLast(LIRExpression::Tuple {
                        elements: vec![
                            LIRExpression::Structure {
                                name: format!("NodeState"),
                                fields: vec![
                                    FieldExpression {
                                        name: format!("mem_i"),
                                        expression: LIRExpression::Binary {
                                            left: Box::new(LIRExpression::Identifier {
                                                identifier: format!("o"),
                                            }),
                                            operator: BinaryOperator::Add,
                                            right: Box::new(LIRExpression::Literal {
                                                literal: Constant::Integer(1),
                                            }),
                                        },
                                    },
                                    FieldExpression {
                                        name: format!("called_node_state"),
                                        expression: LIRExpression::Identifier {
                                            identifier: format!("new_called_node_state"),
                                        },
                                    },
                                ],
                            },
                            LIRExpression::Binary {
                                left: Box::new(LIRExpression::Identifier {
                                    identifier: format!("o"),
                                }),
                                operator: BinaryOperator::Add,
                                right: Box::new(LIRExpression::Identifier {
                                    identifier: format!("y"),
                                }),
                            },
                        ],
                    }),
                ],
            },
        };
        assert_eq!(lir_from_mir(init), control)
    }
}
