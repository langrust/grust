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
        Expression::Array { elements } => {
            let elements = elements
                .into_iter()
                .map(|expression| lir_from_mir(expression))
                .collect();
            LIRExpression::Array { elements }
        }
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

#[cfg(test)]
mod lir_from_mir {
    use crate::ast::pattern::Pattern;
    use crate::common::constant::Constant;
    use crate::common::location::Location;
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::expression::lir_from_mir;
    use crate::lir::block::Block as LIRBlock;
    use crate::lir::expression::{Arm, Expression as LIRExpression, FieldExpression};
    use crate::lir::pattern::Pattern as LIRPattern;
    use crate::lir::r#type::Type as LIRType;
    use crate::lir::statement::r#let::Let;
    use crate::lir::statement::Statement as LIRStatement;
    use crate::mir::block::Block;
    use crate::mir::expression::Expression;
    use crate::mir::statement::Statement;

    #[test]
    fn should_create_lir_literal_from_mir_literal() {
        let expression = Expression::Literal {
            literal: Constant::Integer(1),
        };
        let control = LIRExpression::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_identifier_from_mir_identifier() {
        let expression = Expression::Identifier {
            identifier: String::from("x"),
        };
        let control = LIRExpression::Identifier {
            identifier: String::from("x"),
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_field_access_to_self_from_mir_memory_access() {
        let expression = Expression::MemoryAccess {
            identifier: String::from("mem_x"),
        };
        let control = LIRExpression::FieldAccess {
            expression: Box::new(LIRExpression::Identifier {
                identifier: String::from("self"),
            }),
            field: String::from("mem_x"),
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_structure_from_mir_structure() {
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
        let control = LIRExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                FieldExpression {
                    name: String::from("x"),
                    expression: LIRExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                FieldExpression {
                    name: String::from("y"),
                    expression: LIRExpression::Literal {
                        literal: Constant::Integer(2),
                    },
                },
            ],
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_array_from_mir_array() {
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
        let control = LIRExpression::Array {
            elements: vec![
                LIRExpression::Literal {
                    literal: Constant::Integer(1),
                },
                LIRExpression::Literal {
                    literal: Constant::Integer(2),
                },
            ],
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_block_from_mir_block() {
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
        let control = LIRExpression::Block {
            block: LIRBlock {
                statements: vec![
                    LIRStatement::Let(Let {
                        pattern: LIRPattern::Identifier {
                            reference: false,
                            mutable: false,
                            identifier: String::from("x"),
                        },
                        expression: LIRExpression::Literal {
                            literal: Constant::Integer(1),
                        },
                    }),
                    LIRStatement::ExpressionLast(LIRExpression::Identifier {
                        identifier: String::from("x"),
                    }),
                ],
            },
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_function_call_from_mir_function_call() {
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
        let control = LIRExpression::FunctionCall {
            function: Box::new(LIRExpression::Identifier {
                identifier: String::from("foo"),
            }),
            arguments: vec![
                LIRExpression::Identifier {
                    identifier: String::from("a"),
                },
                LIRExpression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_method_call_from_mir_node_call() {
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
        let control = LIRExpression::MethodCall {
            receiver: Box::new(LIRExpression::FieldAccess {
                expression: Box::new(LIRExpression::Identifier {
                    identifier: String::from("self"),
                }),
                field: String::from("node_state"),
            }),
            method: String::from("step"),
            arguments: vec![LIRExpression::Structure {
                name: String::from("NodeInput"),
                fields: vec![FieldExpression {
                    name: String::from("i"),
                    expression: LIRExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                }],
            }],
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_field_access_from_mir_field_access() {
        let expression = Expression::FieldAccess {
            expression: Box::new(Expression::Identifier {
                identifier: String::from("my_point"),
            }),
            field: String::from("x"),
        };
        let control = LIRExpression::FieldAccess {
            expression: Box::new(LIRExpression::Identifier {
                identifier: String::from("my_point"),
            }),
            field: String::from("x"),
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_closure_from_mir_lambda() {
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
        let control = LIRExpression::Closure {
            r#move: false,
            inputs: vec![LIRPattern::Typed {
                pattern: Box::new(LIRPattern::Identifier {
                    reference: false,
                    mutable: false,
                    identifier: String::from("x"),
                }),
                r#type: LIRType::Identifier {
                    identifier: String::from("i64"),
                },
            }],
            output: Some(LIRType::Identifier {
                identifier: String::from("i64"),
            }),
            body: Box::new(LIRExpression::Block {
                block: LIRBlock {
                    statements: vec![
                        LIRStatement::Let(Let {
                            pattern: LIRPattern::Identifier {
                                reference: false,
                                mutable: false,
                                identifier: String::from("y"),
                            },
                            expression: LIRExpression::Identifier {
                                identifier: String::from("x"),
                            },
                        }),
                        LIRStatement::ExpressionLast(LIRExpression::Identifier {
                            identifier: String::from("y"),
                        }),
                    ],
                },
            }),
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_ifthenelse_from_mir_ifthenelse() {
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
        let control = LIRExpression::IfThenElse {
            condition: Box::new(LIRExpression::Identifier {
                identifier: String::from("test"),
            }),
            then_branch: LIRBlock {
                statements: vec![LIRStatement::ExpressionLast(LIRExpression::Literal {
                    literal: Constant::Integer(1),
                })],
            },
            else_branch: Some(LIRBlock {
                statements: vec![LIRStatement::ExpressionLast(LIRExpression::Literal {
                    literal: Constant::Integer(0),
                })],
            }),
        };
        assert_eq!(lir_from_mir(expression), control)
    }

    #[test]
    fn should_create_lir_match_from_mir_match() {
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
        let control = LIRExpression::Match {
            matched: Box::new(LIRExpression::Identifier {
                identifier: String::from("my_color"),
            }),
            arms: vec![
                Arm {
                    pattern: LIRPattern::Literal {
                        literal: Constant::Enumeration(String::from("Color"), format!("Blue")),
                    },
                    guard: None,
                    body: LIRExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                Arm {
                    pattern: LIRPattern::Literal {
                        literal: Constant::Enumeration(String::from("Color"), format!("Green")),
                    },
                    guard: None,
                    body: LIRExpression::Literal {
                        literal: Constant::Integer(0),
                    },
                },
            ],
        };
        assert_eq!(lir_from_mir(expression), control)
    }
}
