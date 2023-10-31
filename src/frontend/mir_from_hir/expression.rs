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
            arguments: inputs.into_iter().map(mir_from_hir).collect(),
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
            elements: elements.into_iter().map(mir_from_hir).collect(),
        },
        Expression::Match {
            expression, arms, ..
        } => MIRExpression::Match {
            matched: Box::new(mir_from_hir(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, expression)| {
                    (pattern, guard.map(mir_from_hir), mir_from_hir(expression))
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

#[cfg(test)]
mod mir_from_hir {
    use crate::{
        ast::{expression::Expression as ASTExpression, pattern::Pattern},
        common::{constant::Constant, location::Location, r#type::Type},
        frontend::mir_from_hir::expression::mir_from_hir,
        mir::expression::Expression,
    };

    #[test]
    fn should_transform_ast_constant_into_mir_literal() {
        let expression = ASTExpression::Constant {
            constant: Constant::Integer(1),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_call_into_mir_identifier() {
        let expression = ASTExpression::Call {
            id: format!("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Identifier {
            identifier: format!("x"),
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_application_into_mir_function_call() {
        let expression = ASTExpression::Application {
            function_expression: Box::new(ASTExpression::Call {
                id: format!(" + "),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            }),
            inputs: vec![
                ASTExpression::Call {
                    id: format!("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                ASTExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::FunctionCall {
            function: Box::new(Expression::Identifier {
                identifier: format!(" + "),
            }),
            arguments: vec![
                Expression::Identifier {
                    identifier: format!("x"),
                },
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
            ],
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_structure_into_mir_structure() {
        let expression = ASTExpression::Structure {
            name: format!("Point"),
            fields: vec![
                (
                    format!("x"),
                    ASTExpression::Call {
                        id: format!("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    format!("y"),
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Structure(format!("Point"))),
            location: Location::default(),
        };
        let control = Expression::Structure {
            name: format!("Point"),
            fields: vec![
                (
                    format!("x"),
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                ),
                (
                    format!("y"),
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_array_into_mir_array() {
        let expression = ASTExpression::Array {
            elements: vec![ASTExpression::Call {
                id: format!("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Array(Box::new(Type::Integer), 1)),
            location: Location::default(),
        };
        let control = Expression::Array {
            elements: vec![Expression::Identifier {
                identifier: format!("x"),
            }],
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_match_into_mir_match() {
        let expression = ASTExpression::Match {
            expression: Box::new(ASTExpression::Call {
                id: format!("p"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: format!("Point"),
                        fields: vec![
                            (
                                format!("x"),
                                Pattern::Identifier {
                                    name: format!("x"),
                                    location: Location::default(),
                                },
                            ),
                            (
                                format!("y"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    ASTExpression::Call {
                        id: format!("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    Pattern::Default {
                        location: Location::default(),
                    },
                    None,
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: format!("p"),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: format!("Point"),
                        fields: vec![
                            (
                                format!("x"),
                                Pattern::Identifier {
                                    name: format!("x"),
                                    location: Location::default(),
                                },
                            ),
                            (
                                format!("y"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                ),
                (
                    Pattern::Default {
                        location: Location::default(),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_when_into_mir_match() {
        let expression = ASTExpression::When {
            id: format!("x"),
            option: Box::new(ASTExpression::Call {
                id: format!("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            present: Box::new(ASTExpression::Application {
                function_expression: Box::new(ASTExpression::Call {
                    id: format!(" + "),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                }),
                inputs: vec![
                    ASTExpression::Call {
                        id: format!("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ],
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            default: Box::new(ASTExpression::Constant {
                constant: Constant::Integer(1),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: format!("x"),
            }),
            arms: vec![
                (
                    Pattern::Some {
                        pattern: Box::new(Pattern::Identifier {
                            name: format!("x"),
                            location: Location::default(),
                        }),
                        location: Location::default(),
                    },
                    None,
                    Expression::FunctionCall {
                        function: Box::new(Expression::Identifier {
                            identifier: format!(" + "),
                        }),
                        arguments: vec![
                            Expression::Identifier {
                                identifier: format!("x"),
                            },
                            Expression::Literal {
                                literal: Constant::Integer(1),
                            },
                        ],
                    },
                ),
                (
                    Pattern::None {
                        location: Location::default(),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_abstraction_into_mir_node_call() {
        let expression = ASTExpression::TypedAbstraction {
            inputs: vec![(format!("x"), Type::Integer)],
            expression: Box::new(ASTExpression::Application {
                function_expression: Box::new(ASTExpression::Call {
                    id: format!(" + "),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                }),
                inputs: vec![
                    ASTExpression::Call {
                        id: format!("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ],
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
            location: Location::default(),
        };
        let control = Expression::Lambda {
            inputs: vec![(format!("x"), Type::Integer)],
            output: Type::Integer,
            body: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    identifier: format!(" + "),
                }),
                arguments: vec![
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ],
            }),
        };
        assert_eq!(mir_from_hir(expression), control)
    }
}
