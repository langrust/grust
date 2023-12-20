use crate::backend::rust_ast_from_lir::expression::rust_ast_from_lir as expression_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::statement::rust_ast_from_lir as statement_rust_ast_from_lir;
use crate::common::convert_case::camel_case;
use crate::lir::item::node_file::state::step::{StateElementStep, Step};
use crate::rust_ast::block::Block;
use crate::rust_ast::expression::Expression;
use crate::rust_ast::item::implementation::AssociatedItem;
use crate::rust_ast::item::signature::{Receiver, Signature};
use crate::rust_ast::r#type::Type as RustASTType;
use crate::rust_ast::statement::Statement;

/// Transform LIR step into RustAST implementation method.
pub fn rust_ast_from_lir(step: Step) -> AssociatedItem {
    let signature = Signature {
        public_visibility: true,
        name: String::from("step"),
        receiver: Some(Receiver {
            reference: true,
            mutable: true,
        }),
        inputs: vec![(
            String::from("input"),
            RustASTType::Identifier {
                identifier: camel_case(&format!("{}Input", step.node_name)),
            },
        )],
        output: type_rust_ast_from_lir(step.output_type),
    };
    let mut statements = step
        .body
        .into_iter()
        .map(statement_rust_ast_from_lir)
        .collect::<Vec<_>>();

    let mut fields_update = step
        .state_elements_step
        .into_iter()
        .map(
            |StateElementStep {
                 identifier,
                 expression,
             }| {
                let field_acces = Expression::FieldAccess {
                    expression: Box::new(Expression::Identifier {
                        identifier: "self".to_string(),
                    }),
                    field: identifier,
                };
                Statement::ExpressionIntern(Expression::Assignement {
                    left: Box::new(field_acces),
                    right: Box::new(expression_rust_ast_from_lir(expression)),
                })
            },
        )
        .collect::<Vec<_>>();

    let output_statement =
        Statement::ExpressionLast(expression_rust_ast_from_lir(step.output_expression));

    statements.append(&mut fields_update);
    statements.push(output_statement);

    let body = Block { statements };
    AssociatedItem::AssociatedMethod { signature, body }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::node_file::state::step::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::lir::expression::Expression;
    use crate::lir::item::node_file::state::step::{StateElementStep, Step};
    use crate::lir::statement::Statement;
    use crate::rust_ast::block::Block;
    use crate::rust_ast::expression::Expression as RustASTExpression;
    use crate::rust_ast::item::implementation::AssociatedItem;
    use crate::rust_ast::item::signature::{Receiver, Signature};
    use crate::rust_ast::pattern::Pattern;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::rust_ast::statement::r#let::Let;
    use crate::rust_ast::statement::Statement as RustASTStatement;

    #[test]
    fn should_create_rust_ast_associated_method_from_lir_node_init() {
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
                Statement::Let {
                    identifier: format!("y"),
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
                    reference: true,
                    mutable: true,
                }),
                inputs: vec![(
                    format!("input"),
                    RustASTType::Identifier {
                        identifier: format!("NodeInput"),
                    },
                )],
                output: RustASTType::Identifier {
                    identifier: format!("i64"),
                },
            },
            body: Block {
                statements: vec![
                    RustASTStatement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: false,
                            identifier: format!("o"),
                        },
                        expression: RustASTExpression::FieldAccess {
                            expression: Box::new(RustASTExpression::Identifier {
                                identifier: format!("self"),
                            }),
                            field: format!("mem_i"),
                        },
                    }),
                    RustASTStatement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: false,
                            identifier: format!("y"),
                        },
                        expression: RustASTExpression::MethodCall {
                            receiver: Box::new(RustASTExpression::FieldAccess {
                                expression: Box::new(RustASTExpression::Identifier {
                                    identifier: format!("self"),
                                }),
                                field: format!("called_node_state"),
                            }),
                            method: format!("step"),
                            arguments: vec![RustASTExpression::Structure {
                                name: format!("CalledNodeInput"),
                                fields: vec![],
                            }],
                        },
                    }),
                    RustASTStatement::ExpressionIntern(RustASTExpression::Assignement {
                        left: Box::new(RustASTExpression::FieldAccess {
                            expression: Box::new(RustASTExpression::Identifier {
                                identifier: format!("self"),
                            }),
                            field: format!("mem_i"),
                        }),
                        right: Box::new(RustASTExpression::Binary {
                            left: Box::new(RustASTExpression::Identifier {
                                identifier: format!("o"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(RustASTExpression::Literal {
                                literal: Constant::Integer(1),
                            }),
                        }),
                    }),
                    RustASTStatement::ExpressionIntern(RustASTExpression::Assignement {
                        left: Box::new(RustASTExpression::FieldAccess {
                            expression: Box::new(RustASTExpression::Identifier {
                                identifier: format!("self"),
                            }),
                            field: format!("called_node_state"),
                        }),
                        right: Box::new(RustASTExpression::Identifier {
                            identifier: format!("new_called_node_state"),
                        }),
                    }),
                    RustASTStatement::ExpressionLast(RustASTExpression::Binary {
                        left: Box::new(RustASTExpression::Identifier {
                            identifier: format!("o"),
                        }),
                        operator: BinaryOperator::Add,
                        right: Box::new(RustASTExpression::Identifier {
                            identifier: format!("y"),
                        }),
                    }),
                ],
            },
        };
        assert_eq!(rust_ast_from_lir(init), control)
    }
}
