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
            id,
            node,
            signal,
            inputs,
            ..
        } => Expression::NodeCall {
            node_identifier: id.clone().unwrap(),
            input_name: node.clone() + &signal + "Input",
            input_fields: inputs
                .into_iter()
                .map(|(id, expression)| (id, mir_from_hir(expression)))
                .collect(),
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod mir_from_hir {
    use crate::{
        ast::{expression::Expression as ASTExpression, pattern::Pattern},
        common::{constant::Constant, location::Location, r#type::Type},
        frontend::mir_from_hir::stream_expression::mir_from_hir,
        hir::{dependencies::Dependencies, stream_expression::StreamExpression},
        mir::expression::Expression,
    };

    #[test]
    fn should_transform_hir_constant_into_mir_literal() {
        let expression = StreamExpression::Constant {
            constant: Constant::Integer(1),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };
        let control = Expression::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_signal_call_into_mir_identifier() {
        let expression = StreamExpression::SignalCall {
            id: format!("x"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = Expression::Identifier {
            identifier: format!("x"),
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_map_application_into_mir_function_call() {
        let expression = StreamExpression::MapApplication {
            function_expression: ASTExpression::Call {
                id: format!(" + "),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    id: format!("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
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
    fn should_transform_hir_structure_into_mir_structure() {
        let expression = StreamExpression::Structure {
            name: format!("Point"),
            fields: vec![
                (
                    format!("x"),
                    StreamExpression::SignalCall {
                        id: format!("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                    },
                ),
                (
                    format!("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
            ],
            typing: Type::Structure(format!("Point")),
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
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
    fn should_transform_hir_array_into_mir_array() {
        let expression = StreamExpression::Array {
            elements: vec![StreamExpression::SignalCall {
                id: format!("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
            }],
            typing: Type::Array(Box::new(Type::Integer), 1),
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = Expression::Array {
            elements: vec![Expression::Identifier {
                identifier: format!("x"),
            }],
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_match_into_mir_match() {
        let expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: format!("p"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("p"), 0)]),
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
                    vec![],
                    StreamExpression::SignalCall {
                        id: format!("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                    },
                ),
                (
                    Pattern::Default {
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("p"), 0)]),
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
    fn should_transform_hir_when_into_mir_match() {
        let expression = StreamExpression::When {
            id: format!("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: format!("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::MapApplication {
                function_expression: ASTExpression::Call {
                    id: format!(" + "),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: format!("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("p"), 0)]),
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
    fn should_transform_hir_unitary_node_application_into_mir_node_call() {
        let expression = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_nodeox")),
            node: format!("my_node"),
            signal: format!("o"),
            inputs: vec![
                (
                    format!("i"),
                    StreamExpression::SignalCall {
                        id: format!("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                    },
                ),
                (
                    format!("j"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = Expression::NodeCall {
            node_identifier: format!("my_nodeox"),
            input_name: format!("my_nodeoInput"),
            input_fields: vec![
                (
                    format!("i"),
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                ),
                (
                    format!("j"),
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(mir_from_hir(expression), control)
    }
}
