use crate::{
    ast::{expression::Expression, pattern::Pattern},
    common::{operator::OtherOperator, scope::Scope},
    hir::{signal::Signal, stream_expression::StreamExpression},
    mir::{block::Block, expression::Expression as MIRExpression, statement::Statement},
};

use super::{
    equation::mir_from_hir as equation_mir_from_hir,
    expression::mir_from_hir as expression_mir_from_hir,
};

/// Transform HIR stream expression into MIR expression.
pub fn mir_from_hir(stream_expression: StreamExpression) -> MIRExpression {
    match stream_expression {
        StreamExpression::Constant { constant, .. } => MIRExpression::Literal { literal: constant },
        StreamExpression::SignalCall {
            signal:
                Signal {
                    id,
                    scope: Scope::Input,
                    ..
                },
            ..
        } => MIRExpression::InputAccess { identifier: id },
        StreamExpression::SignalCall {
            signal:
                Signal {
                    id,
                    scope: Scope::Memory,
                    ..
                },
            ..
        } => MIRExpression::MemoryAccess { identifier: id },
        StreamExpression::SignalCall {
            signal: Signal { id, .. },
            ..
        } => MIRExpression::Identifier { identifier: id },
        StreamExpression::MapApplication {
            function_expression,
            mut inputs,
            ..
        } => match function_expression {
            Expression::Call { id, .. } if OtherOperator::IfThenElse.to_string() == id => {
                assert!(inputs.len() == 3);
                let else_branch = mir_from_hir(inputs.pop().unwrap());
                let then_branch = mir_from_hir(inputs.pop().unwrap());
                let condition = mir_from_hir(inputs.pop().unwrap());
                MIRExpression::IfThenElse {
                    condition: Box::new(condition),
                    then_branch: Block {
                        statements: vec![Statement::ExpressionLast {
                            expression: then_branch,
                        }],
                    },
                    else_branch: Block {
                        statements: vec![Statement::ExpressionLast {
                            expression: else_branch,
                        }],
                    },
                }
            }
            _ => {
                let arguments = inputs.into_iter().map(mir_from_hir).collect();
                MIRExpression::FunctionCall {
                    function: Box::new(expression_mir_from_hir(function_expression)),
                    arguments,
                }
            }
        },
        StreamExpression::Structure { name, fields, .. } => MIRExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(id, expression)| (id, mir_from_hir(expression)))
                .collect(),
        },
        StreamExpression::Array { elements, .. } => MIRExpression::Array {
            elements: elements.into_iter().map(mir_from_hir).collect(),
        },
        StreamExpression::Match {
            expression, arms, ..
        } => MIRExpression::Match {
            matched: Box::new(mir_from_hir(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, body, expression)| {
                    (
                        pattern,
                        guard.map(mir_from_hir),
                        if body.is_empty() {
                            mir_from_hir(expression)
                        } else {
                            let mut statements = body
                                .into_iter()
                                .map(equation_mir_from_hir)
                                .collect::<Vec<_>>();
                            statements.push(Statement::ExpressionLast {
                                expression: mir_from_hir(expression),
                            });
                            MIRExpression::Block {
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
        StreamExpression::UnitaryNodeApplication {
            id,
            node,
            signal,
            inputs,
            ..
        } => MIRExpression::NodeCall {
            node_identifier: id.unwrap(),
            input_name: node + &signal + "Input",
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
        common::{
            constant::Constant, location::Location, operator::OtherOperator, r#type::Type,
            scope::Scope,
        },
        frontend::mir_from_hir::stream_expression::mir_from_hir,
        hir::{dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression},
        mir::{block::Block, expression::Expression, statement::Statement},
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
    fn should_transform_hir_local_signal_call_into_mir_identifier() {
        let expression = StreamExpression::SignalCall {
            signal: Signal {
                id: format!("x"),
                scope: Scope::Local,
            },
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
    fn should_transform_hir_output_signal_call_into_mir_identifier() {
        let expression = StreamExpression::SignalCall {
            signal: Signal {
                id: format!("o"),
                scope: Scope::Output,
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("o"), 0)]),
        };
        let control = Expression::Identifier {
            identifier: format!("o"),
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_input_signal_call_into_mir_input_access() {
        let expression = StreamExpression::SignalCall {
            signal: Signal {
                id: format!("i"),
                scope: Scope::Input,
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("i"), 0)]),
        };
        let control = Expression::InputAccess {
            identifier: format!("i"),
        };
        assert_eq!(mir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_memory_signal_call_into_mir_memory_access() {
        let expression = StreamExpression::SignalCall {
            signal: Signal {
                id: format!("mem_i"),
                scope: Scope::Memory,
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("mem_i"), 0)]),
        };
        let control = Expression::MemoryAccess {
            identifier: format!("mem_i"),
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
                    signal: Signal {
                        id: format!("x"),
                        scope: Scope::Local,
                    },
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
    fn should_transform_hir_map_application_of_if_then_else_into_mir_if_then_else() {
        let expression = StreamExpression::MapApplication {
            function_expression: ASTExpression::Call {
                id: OtherOperator::IfThenElse.to_string(),
                typing: Some(Type::Abstract(
                    vec![Type::Boolean, Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: format!("test"),
                        scope: Scope::Local,
                    },
                    typing: Type::Boolean,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(format!("test"), 0)]),
                },
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: format!("x"),
                        scope: Scope::Local,
                    },
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
            dependencies: Dependencies::from(vec![(format!("test"), 0), (format!("x"), 0)]),
        };
        let control = Expression::IfThenElse {
            condition: Box::new(Expression::Identifier {
                identifier: format!("test"),
            }),
            then_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Identifier {
                        identifier: format!("x"),
                    },
                }],
            },
            else_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                }],
            },
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
                        signal: Signal {
                            id: format!("x"),
                            scope: Scope::Local,
                        },
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
                signal: Signal {
                    id: format!("x"),
                    scope: Scope::Local,
                },
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
                signal: Signal {
                    id: format!("p"),
                    scope: Scope::Local,
                },
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
                        signal: Signal {
                            id: format!("x"),
                            scope: Scope::Local,
                        },
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
                signal: Signal {
                    id: format!("x"),
                    scope: Scope::Local,
                },
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
                        signal: Signal {
                            id: format!("x"),
                            scope: Scope::Local,
                        },
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
                        signal: Signal {
                            id: format!("x"),
                            scope: Scope::Local,
                        },
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
