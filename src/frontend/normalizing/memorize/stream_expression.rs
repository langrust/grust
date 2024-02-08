use crate::common::scope::Scope;
use crate::hir::term::Contract;
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
    /// node call `memmy_node_o_: (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        signal_name: &String,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
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
                let memory_signal = Signal {
                    id: memory_id.clone(),
                    scope: Scope::Memory,
                };
                contract.rename(signal_name, memory_signal.clone());
                *self = StreamExpression::SignalCall {
                    signal: memory_signal,
                    typing: typing.clone(),
                    location: location.clone(),
                    dependencies: Dependencies::from(vec![(memory_id, 0)]),
                }
            }
            StreamExpression::FunctionApplication {
                inputs,
                dependencies,
                ..
            } => {
                inputs.iter_mut().for_each(|expression| {
                    expression.memorize(signal_name, identifier_creator, memory, contract)
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
                    expression.memorize(signal_name, identifier_creator, memory, contract)
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
                    expression.memorize(signal_name, identifier_creator, memory, contract)
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
                expression.memorize(signal_name, identifier_creator, memory, contract);
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(_, bound_expression, equations, expression)| {
                        if let Some(expression) = bound_expression.as_mut() {
                            expression.memorize(signal_name, identifier_creator, memory, contract)
                        };
                        equations.iter_mut().for_each(|equation| {
                            equation.memorize(identifier_creator, memory, contract)
                        });
                        expression.memorize(signal_name, identifier_creator, memory, contract)
                    });
                let mut arms_dependencies = arms
                    .iter()
                    .flat_map(|(pattern, bound, _, matched_expression)| {
                        // get local signals defined in pattern
                        let local_signals = pattern.local_identifiers();

                        // remove identifiers created by the pattern from the dependencies
                        let mut bound_dependencies = bound.as_ref().map_or(vec![], |expression| {
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
                    .collect::<Vec<_>>();

                expression_dependencies.append(&mut arms_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);
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
                option.memorize(signal_name, identifier_creator, memory, contract);
                present_body
                    .iter_mut()
                    .for_each(|equation| equation.memorize(identifier_creator, memory, contract));
                present.memorize(signal_name, identifier_creator, memory, contract);
                default_body
                    .iter_mut()
                    .for_each(|equation| equation.memorize(identifier_creator, memory, contract));
                default.memorize(signal_name, identifier_creator, memory, contract);

                let mut option_dependencies = option.get_dependencies().clone();
                let mut present_dependencies = present.get_dependencies().clone();
                let mut default_dependencies = default.get_dependencies().clone();
                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);

                *dependencies = Dependencies::from(option_dependencies);
            }
            StreamExpression::FieldAccess {
                expression,
                dependencies,
                ..
            } => {
                expression.memorize(signal_name, identifier_creator, memory, contract);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            StreamExpression::TupleElementAccess {
                expression,
                dependencies,
                ..
            } => {
                expression.memorize(signal_name, identifier_creator, memory, contract);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            StreamExpression::Map {
                expression,
                dependencies,
                ..
            } => {
                expression.memorize(signal_name, identifier_creator, memory, contract);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            StreamExpression::Fold {
                expression,
                initialization_expression,
                ref mut dependencies,
                ..
            } => {
                expression.memorize(signal_name, identifier_creator, memory, contract);
                initialization_expression.memorize(
                    signal_name,
                    identifier_creator,
                    memory,
                    contract,
                );

                // get matched expressions dependencies
                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut initialization_expression_dependencies =
                    expression.get_dependencies().clone();
                expression_dependencies.append(&mut initialization_expression_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
            }
            StreamExpression::Sort {
                expression,
                dependencies,
                ..
            } => {
                expression.memorize(signal_name, identifier_creator, memory, contract);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            StreamExpression::Constant { .. } | StreamExpression::SignalCall { .. } => (),
            StreamExpression::Zip {
                arrays,
                dependencies,
                ..
            } => {
                arrays.iter_mut().for_each(|array| {
                    array.memorize(signal_name, identifier_creator, memory, contract)
                });

                *dependencies = Dependencies::from(
                    arrays
                        .iter()
                        .flat_map(|array| array.get_dependencies().clone())
                        .collect(),
                );
            }
        }
    }
}

#[cfg(test)]
mod memorize {
    use std::collections::HashSet;

    use crate::ast::expression::Expression;
    use crate::common::scope::Scope;
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::term::Contract;
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

        let mut expression = StreamExpression::FunctionApplication {
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
        expression.memorize(
            &String::from("x"),
            &mut identifier_creator,
            &mut memory,
            &mut Contract::default(),
        );

        let mut control = Memory::new();
        control.add_buffer(
            String::from("mem_x"),
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

        let control = StreamExpression::FunctionApplication {
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
                        id: String::from("mem_x"),
                        scope: Scope::Memory,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("mem_x"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![
                (String::from("s"), 0),
                (String::from("mem_x"), 0),
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
            id: Some(format!("my_node_o_y")),
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
        expression.memorize(
            &String::from("y"),
            &mut identifier_creator,
            &mut memory,
            &mut Contract::default(),
        );

        let mut control = Memory::new();
        control.add_called_node(
            String::from("my_node_o_y"),
            String::from("my_node"),
            String::from("o"),
        );
        assert_eq!(memory, control);

        let control = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_node_o_y")),
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
