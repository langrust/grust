use crate::common::scope::Scope;
use crate::hir::{
    dependencies::Dependencies, identifier_creator::IdentifierCreator, memory::Memory,
    signal::Signal, stream_expression::StreamExpression,
};

impl StreamExpression {
    /// Increment memory with expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// An expression `0 fby v` increments memory with the buffer
    /// `mem: int = 0 fby v;` and becomes a call to `mem`.
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the
    /// node call `memmy_nodeo: (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        signal_name: &String,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
    ) {
        match self {
            StreamExpression::FollowedBy {
                constant,
                expression,
                typing,
                location,
                ..
            } => {
                let memory_id = identifier_creator.new_identifier(
                    String::from("mem"),
                    signal_name.clone(),
                    String::from(""),
                );
                memory.add_buffer(memory_id.clone(), constant.clone(), *expression.clone());
                *self = StreamExpression::SignalCall {
                    signal: Signal {
                        id: memory_id.clone(),
                        scope: Scope::Memory,
                    },
                    typing: typing.clone(),
                    location: location.clone(),
                    dependencies: Dependencies::from(vec![(memory_id, 0)]),
                }
            }
            StreamExpression::MapApplication {
                inputs,
                dependencies,
                ..
            } => {
                inputs.iter_mut().for_each(|expression| {
                    expression.memorize(signal_name, identifier_creator, memory)
                });

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication {
                id,
                inputs,
                dependencies,
                node,
                signal,
                ..
            } => {
                memory.add_called_node(id.clone().unwrap(), node.clone(), signal.clone());

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            StreamExpression::Structure {
                fields,
                dependencies,
                ..
            } => {
                fields.iter_mut().for_each(|(_, expression)| {
                    expression.memorize(signal_name, identifier_creator, memory)
                });

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            StreamExpression::Array {
                elements,
                dependencies,
                ..
            } => {
                elements.iter_mut().for_each(|expression| {
                    expression.memorize(signal_name, identifier_creator, memory)
                });

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            StreamExpression::Match {
                expression,
                arms,
                dependencies,
                ..
            } => {
                expression.memorize(signal_name, identifier_creator, memory);
                arms.iter_mut()
                    .for_each(|(_, bound_expression, equations, expression)| {
                        if let Some(expression) = bound_expression.as_mut() {
                            expression.memorize(signal_name, identifier_creator, memory)
                        };
                        equations
                            .iter_mut()
                            .for_each(|equation| equation.memorize(identifier_creator, memory));
                        expression.memorize(signal_name, identifier_creator, memory)
                    });

                *dependencies = Dependencies::from(
                    arms.iter()
                        .flat_map(|(pattern, bound, _, matched_expression)| {
                            // get local signals defined in pattern
                            let local_signals = pattern.local_identifiers();

                            // remove identifiers created by the pattern from the dependencies
                            let mut bound_dependencies =
                                bound.as_ref().map_or(vec![], |expression| {
                                    expression
                                        .get_dependencies()
                                        .clone()
                                        .into_iter()
                                        .filter(|(signal, _)| !local_signals.contains(signal))
                                        .collect()
                                });
                            let mut matched_expression_dependencies = matched_expression
                                .get_dependencies()
                                .clone()
                                .into_iter()
                                .filter(|(signal, _)| !local_signals.contains(signal))
                                .collect::<Vec<_>>();
                            matched_expression_dependencies.append(&mut bound_dependencies);
                            matched_expression_dependencies
                        })
                        .collect(),
                );
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                dependencies,
                ..
            } => {
                option.memorize(signal_name, identifier_creator, memory);
                present_body
                    .iter_mut()
                    .for_each(|equation| equation.memorize(identifier_creator, memory));
                present.memorize(signal_name, identifier_creator, memory);
                default_body
                    .iter_mut()
                    .for_each(|equation| equation.memorize(identifier_creator, memory));
                default.memorize(signal_name, identifier_creator, memory);

                let mut option_dependencies = option.get_dependencies().clone();
                let mut present_dependencies = present.get_dependencies().clone();
                let mut default_dependencies = default.get_dependencies().clone();
                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);

                *dependencies = Dependencies::from(option_dependencies);
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod memorize {
    use std::collections::HashSet;

    use crate::ast::expression::Expression;
    use crate::common::scope::Scope;
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::{
        dependencies::Dependencies, identifier_creator::IdentifierCreator, memory::Memory,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_memorize_followed_by() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };
        let mut memory = Memory::new();

        let mut expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("s"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                },
                StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("v"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    }),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 1)]),
        };
        expression.memorize(&String::from("x"), &mut identifier_creator, &mut memory);

        let mut control = Memory::new();
        control.add_buffer(
            String::from("memx"),
            Constant::Integer(0),
            StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("v"),
                    scope: Scope::Input,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
            },
        );
        assert_eq!(memory, control);

        let control = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("s"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                },
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("memx"),
                        scope: Scope::Memory,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("memx"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![
                (String::from("s"), 0),
                (String::from("memx"), 0),
            ]),
        };
        assert_eq!(expression, control);
    }

    #[test]
    fn should_memorize_node_expression() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };
        let mut memory = Memory::new();

        let mut expression = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_nodeoy")),
            node: String::from("my_node"),
            inputs: vec![
                (
                    format!("x"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("s"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                    },
                ),
                (
                    format!("y"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                    },
                ),
            ],
            signal: String::from("o"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![
                (String::from("s"), 0),
                (String::from("x_1"), 0),
            ]),
        };
        expression.memorize(&String::from("y"), &mut identifier_creator, &mut memory);

        let mut control = Memory::new();
        control.add_called_node(
            String::from("my_nodeoy"),
            String::from("my_node"),
            String::from("o"),
        );
        assert_eq!(memory, control);

        let control = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_nodeoy")),
            node: String::from("my_node"),
            inputs: vec![
                (
                    format!("x"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("s"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                    },
                ),
                (
                    format!("y"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                    },
                ),
            ],
            signal: String::from("o"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![
                (String::from("s"), 0),
                (String::from("x_1"), 0),
            ]),
        };
        assert_eq!(expression, control);
    }
}
