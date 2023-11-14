use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    ast::{expression::Expression, pattern::Pattern},
    common::{
        convert_case::camel_case,
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        scope::Scope,
    },
    hir::{signal::Signal, stream_expression::StreamExpression},
    lir::{
        block::Block, expression::Expression as LIRExpression, item::node_file::import::Import,
        statement::Statement,
    },
};

use super::{
    equation::lir_from_hir as equation_lir_from_hir,
    expression::lir_from_hir as expression_lir_from_hir,
};

/// Transform HIR stream expression into LIR expression.
pub fn lir_from_hir(stream_expression: StreamExpression) -> LIRExpression {
    match stream_expression {
        StreamExpression::Constant { constant, .. } => LIRExpression::Literal { literal: constant },
        StreamExpression::SignalCall {
            signal:
                Signal {
                    id,
                    scope: Scope::Input,
                    ..
                },
            ..
        } => LIRExpression::InputAccess { identifier: id },
        StreamExpression::SignalCall {
            signal:
                Signal {
                    id,
                    scope: Scope::Memory,
                    ..
                },
            ..
        } => LIRExpression::MemoryAccess { identifier: id },
        StreamExpression::SignalCall {
            signal: Signal { id, .. },
            ..
        } => LIRExpression::Identifier { identifier: id },
        StreamExpression::MapApplication {
            function_expression,
            mut inputs,
            ..
        } => match function_expression {
            Expression::Call { id, .. } if OtherOperator::IfThenElse.to_string() == id => {
                assert!(inputs.len() == 3);
                let else_branch = lir_from_hir(inputs.pop().unwrap());
                let then_branch = lir_from_hir(inputs.pop().unwrap());
                let condition = lir_from_hir(inputs.pop().unwrap());
                LIRExpression::IfThenElse {
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
                let arguments = inputs.into_iter().map(lir_from_hir).collect();
                LIRExpression::FunctionCall {
                    function: Box::new(expression_lir_from_hir(function_expression)),
                    arguments,
                }
            }
        },
        StreamExpression::Structure { name, fields, .. } => LIRExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(id, expression)| (id, lir_from_hir(expression)))
                .collect(),
        },
        StreamExpression::Array { elements, .. } => LIRExpression::Array {
            elements: elements.into_iter().map(lir_from_hir).collect(),
        },
        StreamExpression::Match {
            expression, arms, ..
        } => LIRExpression::Match {
            matched: Box::new(lir_from_hir(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, body, expression)| {
                    (
                        pattern,
                        guard.map(lir_from_hir),
                        if body.is_empty() {
                            lir_from_hir(expression)
                        } else {
                            let mut statements = body
                                .into_iter()
                                .map(equation_lir_from_hir)
                                .collect::<Vec<_>>();
                            statements.push(Statement::ExpressionLast {
                                expression: lir_from_hir(expression),
                            });
                            LIRExpression::Block {
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
        } => LIRExpression::Match {
            matched: Box::new(lir_from_hir(*option)),
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
                    lir_from_hir(*present),
                ),
                (Pattern::None { location }, None, lir_from_hir(*default)),
            ],
        },
        StreamExpression::UnitaryNodeApplication {
            id,
            node,
            signal,
            inputs,
            ..
        } => LIRExpression::NodeCall {
            node_identifier: id.unwrap(),
            input_name: camel_case(&format!("{node}_{signal}Input")),
            input_fields: inputs
                .into_iter()
                .map(|(id, expression)| (id, lir_from_hir(expression)))
                .collect(),
        },
        _ => unreachable!(),
    }
}

impl StreamExpression {
    /// Get imports induced by expression.
    pub fn get_imports(&self) -> Vec<Import> {
        match self {
            StreamExpression::SignalCall { .. } => vec![],
            StreamExpression::Constant { constant, .. } => constant.get_imports(),
            StreamExpression::FollowedBy {
                expression,
                constant,
                ..
            } => {
                let mut imports = expression.get_imports();
                let mut constant_import = constant.get_imports();
                imports.append(&mut constant_import);
                imports.into_iter().unique().collect()
            }
            StreamExpression::MapApplication {
                function_expression,
                inputs,
                ..
            } => {
                let mut imports = inputs
                    .iter()
                    .flat_map(StreamExpression::get_imports)
                    .collect::<Vec<_>>();
                let mut function_import = match function_expression {
                    Expression::Call { id, .. }
                        if BinaryOperator::iter()
                            .find(|binary| &binary.to_string() == id)
                            .is_none()
                            && UnaryOperator::iter()
                                .find(|unary| &unary.to_string() == id)
                                .is_none()
                            && OtherOperator::iter()
                                .find(|other| &other.to_string() == id)
                                .is_none() =>
                    {
                        vec![Import::Function(id.clone())]
                    }
                    _ => function_expression.get_imports(),
                };
                imports.append(&mut function_import);
                imports.into_iter().unique().collect()
            }
            StreamExpression::UnitaryNodeApplication {
                node: node_id,
                signal: signal_id,
                inputs,
                ..
            } => {
                let mut imports = inputs
                    .iter()
                    .flat_map(|(_, expression)| expression.get_imports())
                    .collect::<Vec<_>>();
                let node_name = format!("{node_id}_{signal_id}");
                imports.push(Import::NodeFile(node_name));
                imports.into_iter().unique().collect()
            }
            StreamExpression::Structure { name, fields, .. } => {
                let mut imports = fields
                    .iter()
                    .flat_map(|(_, expression)| expression.get_imports())
                    .collect::<Vec<_>>();
                imports.push(Import::Structure(name.clone()));
                imports.into_iter().unique().collect()
            }
            StreamExpression::Array { elements, .. } => elements
                .iter()
                .flat_map(StreamExpression::get_imports)
                .unique()
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut arms_imports = arms
                    .iter()
                    .flat_map(|(pattern, guard, body, expression)| {
                        let mut pattern_imports = pattern.get_imports();
                        let mut guard_imports =
                            guard.as_ref().map_or(vec![], StreamExpression::get_imports);
                        let mut body_imports = body
                            .iter()
                            .flat_map(|equation| equation.expression.get_imports())
                            .collect();
                        let mut expression_imports = expression.get_imports();

                        let mut imports = vec![];
                        imports.append(&mut pattern_imports);
                        imports.append(&mut guard_imports);
                        imports.append(&mut body_imports);
                        imports.append(&mut expression_imports);
                        imports
                    })
                    .collect();
                let mut expression_imports = expression.get_imports();

                let mut imports = vec![];
                imports.append(&mut arms_imports);
                imports.append(&mut expression_imports);
                imports.into_iter().unique().collect()
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut option_imports = option.get_imports();
                let mut present_body_imports = present_body
                    .iter()
                    .flat_map(|equation| equation.expression.get_imports())
                    .collect();
                let mut present_imports = present.get_imports();
                let mut default_body_imports = default_body
                    .iter()
                    .flat_map(|equation| equation.expression.get_imports())
                    .collect();
                let mut default_imports = default.get_imports();

                let mut imports = vec![];
                imports.append(&mut option_imports);
                imports.append(&mut present_body_imports);
                imports.append(&mut present_imports);
                imports.append(&mut default_body_imports);
                imports.append(&mut default_imports);
                imports.into_iter().unique().collect()
            }
            StreamExpression::NodeApplication { .. } => unreachable!(),
        }
    }
}

#[cfg(test)]
mod lir_from_hir {
    use crate::{
        ast::{expression::Expression as ASTExpression, pattern::Pattern},
        common::{
            constant::Constant, location::Location, operator::OtherOperator, r#type::Type,
            scope::Scope,
        },
        frontend::lir_from_hir::stream_expression::lir_from_hir,
        hir::{dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression},
        lir::{block::Block, expression::Expression, statement::Statement},
    };

    #[test]
    fn should_transform_hir_constant_into_lir_literal() {
        let expression = StreamExpression::Constant {
            constant: Constant::Integer(1),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };
        let control = Expression::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_local_signal_call_into_lir_identifier() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_output_signal_call_into_lir_identifier() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_input_signal_call_into_lir_input_access() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_memory_signal_call_into_lir_memory_access() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_map_application_into_lir_function_call() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_map_application_of_if_then_else_into_lir_if_then_else() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_structure_into_lir_structure() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_array_into_lir_array() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_match_into_lir_match() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_when_into_lir_match() {
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
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_hir_unitary_node_application_into_lir_node_call() {
        let expression = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_node_o_x")),
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
            node_identifier: format!("my_node_o_x"),
            input_name: format!("MyNodeOInput"),
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
        assert_eq!(lir_from_hir(expression), control)
    }
}

#[cfg(test)]
mod get_imports {
    use crate::{
        ast::expression::Expression,
        common::{location::Location, operator::UnaryOperator, r#type::Type, scope::Scope},
        hir::{dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression},
        lir::item::node_file::import::Import,
    };

    #[test]
    fn should_get_function_import_from_function_call_expression() {
        let expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: format!("my_function"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                signal: Signal {
                    id: format!("x"),
                    scope: Scope::Input,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
            }],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = vec![Import::Function(format!("my_function"))];
        assert_eq!(expression.get_imports(), control)
    }

    #[test]
    fn should_not_import_builtin_functions() {
        let expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: UnaryOperator::Neg.to_string(),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                signal: Signal {
                    id: format!("x"),
                    scope: Scope::Input,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
            }],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = vec![];
        assert_eq!(expression.get_imports(), control)
    }

    #[test]
    fn should_get_node_import_from_node_call_expression() {
        let expression = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_node_o_x")),
            node: format!("my_node"),
            signal: format!("o"),
            inputs: vec![(
                format!("i"),
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: format!("x"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                },
            )],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = vec![Import::NodeFile(format!("my_node_o"))];
        assert_eq!(expression.get_imports(), control)
    }

    #[test]
    fn should_not_duplicate_imports() {
        let expression = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_node_o_x")),
            node: format!("my_node"),
            signal: format!("o"),
            inputs: vec![(
                format!("i"),
                StreamExpression::UnitaryNodeApplication {
                    id: Some(format!("my_node_o_x")),
                    node: format!("my_node"),
                    signal: format!("o"),
                    inputs: vec![(
                        format!("i"),
                        StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: format!("my_function"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: format!("my_function"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: format!("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                        },
                    )],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                },
            )],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
        };
        let control = vec![
            Import::Function(format!("my_function")),
            Import::NodeFile(format!("my_node_o")),
        ];
        assert_eq!(expression.get_imports(), control)
    }
}
