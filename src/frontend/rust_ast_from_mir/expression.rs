use crate::common::operator::{BinaryOperator, UnaryOperator};
use crate::rust_ast::expression::{Arm, Expression as RustASTExpression, FieldExpression};
use crate::rust_ast::pattern::Pattern;
use crate::lir::expression::Expression;

use super::{
    block::rust_ast_from_mir as block_rust_ast_from_mir, pattern::rust_ast_from_mir as pattern_rust_ast_from_mir,
    r#type::rust_ast_from_mir as type_rust_ast_from_mir,
};

use strum::IntoEnumIterator;

/// Transform LIR expression into RustAST expression.
pub fn rust_ast_from_mir(expression: Expression) -> RustASTExpression {
    match expression {
        Expression::Literal { literal } => RustASTExpression::Literal { literal },
        Expression::Identifier { identifier } => RustASTExpression::Identifier { identifier },
        Expression::MemoryAccess { identifier } => {
            let self_call = RustASTExpression::Identifier {
                identifier: String::from("self"),
            };
            RustASTExpression::FieldAccess {
                expression: Box::new(self_call),
                field: identifier,
            }
        }
        Expression::InputAccess { identifier } => {
            let input_call = RustASTExpression::Identifier {
                identifier: String::from("input"),
            };
            RustASTExpression::FieldAccess {
                expression: Box::new(input_call),
                field: identifier,
            }
        }
        Expression::Structure { name, fields } => {
            let fields = fields
                .into_iter()
                .map(|(name, expression)| FieldExpression {
                    name,
                    expression: rust_ast_from_mir(expression),
                })
                .collect();
            RustASTExpression::Structure { name, fields }
        }
        Expression::Array { elements } => {
            let elements = elements.into_iter().map(rust_ast_from_mir).collect();
            RustASTExpression::Array { elements }
        }
        Expression::Block { block } => RustASTExpression::Block {
            block: block_rust_ast_from_mir(block),
        },
        Expression::FunctionCall {
            function,
            mut arguments,
        } => match function.as_ref() {
            Expression::Identifier { identifier } => {
                if let Some(binary) =
                    BinaryOperator::iter().find(|binary| binary.to_string() == *identifier)
                {
                    RustASTExpression::Binary {
                        left: Box::new(rust_ast_from_mir(arguments.remove(0))),
                        operator: binary,
                        right: Box::new(rust_ast_from_mir(arguments.remove(0))),
                    }
                } else if let Some(unary) =
                    UnaryOperator::iter().find(|unary| unary.to_string() == *identifier)
                {
                    RustASTExpression::Unary {
                        operator: unary,
                        expression: Box::new(rust_ast_from_mir(arguments.remove(0))),
                    }
                } else {
                    let arguments = arguments.into_iter().map(rust_ast_from_mir).collect();
                    RustASTExpression::FunctionCall {
                        function: Box::new(rust_ast_from_mir(*function)),
                        arguments,
                    }
                }
            }
            _ => {
                let arguments = arguments.into_iter().map(rust_ast_from_mir).collect();
                RustASTExpression::FunctionCall {
                    function: Box::new(rust_ast_from_mir(*function)),
                    arguments,
                }
            }
        },
        Expression::NodeCall {
            node_identifier,
            input_name,
            input_fields,
        } => {
            let self_call = RustASTExpression::Identifier {
                identifier: String::from("self"),
            };
            let receiver = RustASTExpression::FieldAccess {
                expression: Box::new(self_call),
                field: node_identifier,
            };
            let input_fields = input_fields
                .into_iter()
                .map(|(name, expression)| FieldExpression {
                    name,
                    expression: rust_ast_from_mir(expression),
                })
                .collect();
            let argument = RustASTExpression::Structure {
                name: input_name,
                fields: input_fields,
            };
            RustASTExpression::MethodCall {
                receiver: Box::new(receiver),
                method: String::from("step"),
                arguments: vec![argument],
            }
        }
        Expression::FieldAccess { expression, field } => RustASTExpression::FieldAccess {
            expression: Box::new(rust_ast_from_mir(*expression)),
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
                    r#type: type_rust_ast_from_mir(r#type),
                })
                .collect();
            RustASTExpression::Closure {
                r#move: false,
                inputs,
                output: Some(type_rust_ast_from_mir(output)),
                body: Box::new(rust_ast_from_mir(*body)),
            }
        }
        Expression::IfThenElse {
            condition,
            then_branch,
            else_branch,
        } => RustASTExpression::IfThenElse {
            condition: Box::new(rust_ast_from_mir(*condition)),
            then_branch: block_rust_ast_from_mir(then_branch),
            else_branch: Some(block_rust_ast_from_mir(else_branch)),
        },
        Expression::Match { matched, arms } => {
            let arms = arms
                .into_iter()
                .map(|(pattern, guard, body)| Arm {
                    pattern: pattern_rust_ast_from_mir(pattern),
                    guard: guard.map(rust_ast_from_mir),
                    body: rust_ast_from_mir(body),
                })
                .collect();
            RustASTExpression::Match {
                matched: Box::new(rust_ast_from_mir(*matched)),
                arms,
            }
        }
    }
}

#[cfg(test)]
mod rust_ast_from_mir {
    use crate::ast::pattern::Pattern;
    use crate::common::constant::Constant;
    use crate::common::location::Location;
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::frontend::rust_ast_from_mir::expression::rust_ast_from_mir;
    use crate::rust_ast::block::Block as RustASTBlock;
    use crate::rust_ast::expression::{Arm, Expression as RustASTExpression, FieldExpression};
    use crate::rust_ast::pattern::Pattern as RustASTPattern;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::rust_ast::statement::r#let::Let;
    use crate::rust_ast::statement::Statement as RustASTStatement;
    use crate::lir::block::Block;
    use crate::lir::expression::Expression;
    use crate::lir::statement::Statement;

    #[test]
    fn should_create_rust_ast_literal_from_mir_literal() {
        let expression = Expression::Literal {
            literal: Constant::Integer(1),
        };
        let control = RustASTExpression::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_identifier_from_mir_identifier() {
        let expression = Expression::Identifier {
            identifier: String::from("x"),
        };
        let control = RustASTExpression::Identifier {
            identifier: String::from("x"),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_self_from_mir_memory_access() {
        let expression = Expression::MemoryAccess {
            identifier: String::from("mem_x"),
        };
        let control = RustASTExpression::FieldAccess {
            expression: Box::new(RustASTExpression::Identifier {
                identifier: String::from("self"),
            }),
            field: String::from("mem_x"),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_input_from_mir_input_access() {
        let expression = Expression::InputAccess {
            identifier: String::from("i"),
        };
        let control = RustASTExpression::FieldAccess {
            expression: Box::new(RustASTExpression::Identifier {
                identifier: String::from("input"),
            }),
            field: String::from("i"),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_structure_from_mir_structure() {
        let expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
                (
                    String::from("y"),
                    Expression::Literal {
                        literal: Constant::Integer(2),
                    },
                ),
            ],
        };
        let control = RustASTExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                FieldExpression {
                    name: String::from("x"),
                    expression: RustASTExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                FieldExpression {
                    name: String::from("y"),
                    expression: RustASTExpression::Literal {
                        literal: Constant::Integer(2),
                    },
                },
            ],
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_array_from_mir_array() {
        let expression = Expression::Array {
            elements: vec![
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
                Expression::Literal {
                    literal: Constant::Integer(2),
                },
            ],
        };
        let control = RustASTExpression::Array {
            elements: vec![
                RustASTExpression::Literal {
                    literal: Constant::Integer(1),
                },
                RustASTExpression::Literal {
                    literal: Constant::Integer(2),
                },
            ],
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_block_from_mir_block() {
        let expression = Expression::Block {
            block: Block {
                statements: vec![
                    Statement::Let {
                        identifier: String::from("x"),
                        expression: Expression::Literal {
                            literal: Constant::Integer(1),
                        },
                    },
                    Statement::ExpressionLast {
                        expression: Expression::Identifier {
                            identifier: String::from("x"),
                        },
                    },
                ],
            },
        };
        let control = RustASTExpression::Block {
            block: RustASTBlock {
                statements: vec![
                    RustASTStatement::Let(Let {
                        pattern: RustASTPattern::Identifier {
                            reference: false,
                            mutable: false,
                            identifier: String::from("x"),
                        },
                        expression: RustASTExpression::Literal {
                            literal: Constant::Integer(1),
                        },
                    }),
                    RustASTStatement::ExpressionLast(RustASTExpression::Identifier {
                        identifier: String::from("x"),
                    }),
                ],
            },
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_function_call_from_mir_function_call() {
        let expression = Expression::FunctionCall {
            function: Box::new(Expression::Identifier {
                identifier: String::from("foo"),
            }),
            arguments: vec![
                Expression::Identifier {
                    identifier: String::from("a"),
                },
                Expression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };
        let control = RustASTExpression::FunctionCall {
            function: Box::new(RustASTExpression::Identifier {
                identifier: String::from("foo"),
            }),
            arguments: vec![
                RustASTExpression::Identifier {
                    identifier: String::from("a"),
                },
                RustASTExpression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_binary_from_mir_function_call() {
        let expression = Expression::FunctionCall {
            function: Box::new(Expression::Identifier {
                identifier: String::from(" + "),
            }),
            arguments: vec![
                Expression::Identifier {
                    identifier: String::from("a"),
                },
                Expression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };
        let control = RustASTExpression::Binary {
            left: Box::new(RustASTExpression::Identifier {
                identifier: String::from("a"),
            }),
            operator: BinaryOperator::Add,
            right: Box::new(RustASTExpression::Identifier {
                identifier: String::from("b"),
            }),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_method_call_from_mir_node_call() {
        let expression = Expression::NodeCall {
            node_identifier: String::from("node_state"),
            input_name: String::from("NodeInput"),
            input_fields: vec![(
                String::from("i"),
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
            )],
        };
        let control = RustASTExpression::MethodCall {
            receiver: Box::new(RustASTExpression::FieldAccess {
                expression: Box::new(RustASTExpression::Identifier {
                    identifier: String::from("self"),
                }),
                field: String::from("node_state"),
            }),
            method: String::from("step"),
            arguments: vec![RustASTExpression::Structure {
                name: String::from("NodeInput"),
                fields: vec![FieldExpression {
                    name: String::from("i"),
                    expression: RustASTExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                }],
            }],
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_from_mir_field_access() {
        let expression = Expression::FieldAccess {
            expression: Box::new(Expression::Identifier {
                identifier: String::from("my_point"),
            }),
            field: String::from("x"),
        };
        let control = RustASTExpression::FieldAccess {
            expression: Box::new(RustASTExpression::Identifier {
                identifier: String::from("my_point"),
            }),
            field: String::from("x"),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_closure_from_mir_lambda() {
        let expression = Expression::Lambda {
            inputs: vec![(String::from("x"), Type::Integer)],
            output: Type::Integer,
            body: Box::new(Expression::Block {
                block: Block {
                    statements: vec![
                        Statement::Let {
                            identifier: String::from("y"),
                            expression: Expression::Identifier {
                                identifier: String::from("x"),
                            },
                        },
                        Statement::ExpressionLast {
                            expression: Expression::Identifier {
                                identifier: String::from("y"),
                            },
                        },
                    ],
                },
            }),
        };
        let control = RustASTExpression::Closure {
            r#move: false,
            inputs: vec![RustASTPattern::Typed {
                pattern: Box::new(RustASTPattern::Identifier {
                    reference: false,
                    mutable: false,
                    identifier: String::from("x"),
                }),
                r#type: RustASTType::Identifier {
                    identifier: String::from("i64"),
                },
            }],
            output: Some(RustASTType::Identifier {
                identifier: String::from("i64"),
            }),
            body: Box::new(RustASTExpression::Block {
                block: RustASTBlock {
                    statements: vec![
                        RustASTStatement::Let(Let {
                            pattern: RustASTPattern::Identifier {
                                reference: false,
                                mutable: false,
                                identifier: String::from("y"),
                            },
                            expression: RustASTExpression::Identifier {
                                identifier: String::from("x"),
                            },
                        }),
                        RustASTStatement::ExpressionLast(RustASTExpression::Identifier {
                            identifier: String::from("y"),
                        }),
                    ],
                },
            }),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_ifthenelse_from_mir_ifthenelse() {
        let expression = Expression::IfThenElse {
            condition: Box::new(Expression::Identifier {
                identifier: String::from("test"),
            }),
            then_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                }],
            },
            else_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Literal {
                        literal: Constant::Integer(0),
                    },
                }],
            },
        };
        let control = RustASTExpression::IfThenElse {
            condition: Box::new(RustASTExpression::Identifier {
                identifier: String::from("test"),
            }),
            then_branch: RustASTBlock {
                statements: vec![RustASTStatement::ExpressionLast(RustASTExpression::Literal {
                    literal: Constant::Integer(1),
                })],
            },
            else_branch: Some(RustASTBlock {
                statements: vec![RustASTStatement::ExpressionLast(RustASTExpression::Literal {
                    literal: Constant::Integer(0),
                })],
            }),
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }

    #[test]
    fn should_create_rust_ast_match_from_mir_match() {
        let expression = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: String::from("my_color"),
            }),
            arms: vec![
                (
                    Pattern::Constant {
                        constant: Constant::Enumeration(String::from("Color"), format!("Blue")),
                        location: Location::default(),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
                (
                    Pattern::Constant {
                        constant: Constant::Enumeration(String::from("Color"), format!("Green")),
                        location: Location::default(),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(0),
                    },
                ),
            ],
        };
        let control = RustASTExpression::Match {
            matched: Box::new(RustASTExpression::Identifier {
                identifier: String::from("my_color"),
            }),
            arms: vec![
                Arm {
                    pattern: RustASTPattern::Literal {
                        literal: Constant::Enumeration(String::from("Color"), format!("Blue")),
                    },
                    guard: None,
                    body: RustASTExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                Arm {
                    pattern: RustASTPattern::Literal {
                        literal: Constant::Enumeration(String::from("Color"), format!("Green")),
                    },
                    guard: None,
                    body: RustASTExpression::Literal {
                        literal: Constant::Integer(0),
                    },
                },
            ],
        };
        assert_eq!(rust_ast_from_mir(expression), control)
    }
}
