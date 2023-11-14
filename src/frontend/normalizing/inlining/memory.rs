use std::collections::HashMap;

use crate::{
    common::scope::Scope,
    hir::{
        identifier_creator::IdentifierCreator, memory::Memory, signal::Signal,
        stream_expression::StreamExpression,
    },
};

use super::Union;

impl Memory {
    /// Add the buffer and called_node identifier to the identifier creator.
    ///
    /// It will add the buffer and called_node identifier to the identifier creator.
    /// If the identifier already exists, then the new identifer created by
    /// the identifier creator will be added to the renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<String, Union<Signal, StreamExpression>>,
    ) {
        self.buffers.keys().for_each(|id| {
            let new_id =
                identifier_creator.new_identifier(String::new(), id.clone(), String::new());
            if new_id.ne(id) {
                assert!(context_map
                    .insert(
                        id.clone(),
                        Union::I1(Signal {
                            id: new_id,
                            scope: Scope::Memory
                        })
                    )
                    .is_none());
            }
        });
        self.called_nodes.keys().for_each(|id| {
            let new_id =
                identifier_creator.new_identifier(String::new(), id.clone(), String::new());
            if new_id.ne(id) {
                assert!(context_map
                    .insert(
                        id.clone(),
                        Union::I1(Signal {
                            id: new_id,
                            scope: Scope::Memory
                        })
                    )
                    .is_none());
            }
        })
    }

    /// Replace identifier occurence by element in context.
    ///
    /// It will return a new memory where the expression has been modified
    /// according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function
    /// with the equation `z = x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<String, Union<Signal, StreamExpression>>,
    ) -> Memory {
        let buffers = self
            .buffers
            .iter()
            .map(|(buffer_id, buffer)| {
                let mut new_buffer = buffer.clone();
                new_buffer.expression.replace_by_context(context_map);

                if let Some(element) = context_map.get(buffer_id) {
                    match element {
                        Union::I1(Signal { id: new_id, .. })
                        | Union::I2(StreamExpression::SignalCall {
                            signal: Signal { id: new_id, .. },
                            ..
                        }) => (new_id.clone(), new_buffer),
                        Union::I2(_) => unreachable!(),
                    }
                } else {
                    (buffer_id.clone(), new_buffer)
                }
            })
            .collect();

        let called_nodes = self
            .called_nodes
            .iter()
            .map(|(called_node_id, called_node)| {
                if let Some(element) = context_map.get(called_node_id) {
                    match element {
                        Union::I1(Signal { id: new_id, .. })
                        | Union::I2(StreamExpression::SignalCall {
                            signal: Signal { id: new_id, .. },
                            ..
                        }) => (new_id.clone(), called_node.clone()),
                        Union::I2(_) => unreachable!(),
                    }
                } else {
                    (called_node_id.clone(), called_node.clone())
                }
            })
            .collect();

        Memory {
            buffers,
            called_nodes,
        }
    }

    /// Remove called node from memory.
    pub fn remove_called_node(&mut self, called_node_id: &String) {
        self.called_nodes.remove(called_node_id);
    }

    /// Combine two memories.
    pub fn combine(&mut self, other: Memory) {
        self.buffers.extend(other.buffers);
        self.called_nodes.extend(other.called_nodes);
    }
}

#[cfg(test)]
mod add_necessary_renaming {
    use std::collections::HashMap;

    use crate::common::constant::Constant;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::identifier_creator::IdentifierCreator;
    use crate::hir::memory::{Buffer, CalledNode, Memory};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_buffer_id_to_the_identifier_creator_if_id_is_not_used() {
        let memory = Memory {
            buffers: HashMap::from([(
                String::from("mem_y"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };

        let mut context_map = HashMap::from([(
            String::from("x"),
            Union::I1(Signal {
                id: String::from("a"),
                scope: Scope::Local,
            }),
        )]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        memory.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = IdentifierCreator::from(vec![String::from("a"), String::from("mem_y")]);

        assert_eq!(identifier_creator, control)
    }

    #[test]
    fn should_add_buffer_signal_to_the_context_if_id_is_already_used() {
        let memory = Memory {
            buffers: HashMap::from([(
                String::from("a"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };

        let mut context_map = HashMap::from([(
            String::from("x"),
            Union::I1(Signal {
                id: String::from("a"),
                scope: Scope::Local,
            }),
        )]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        memory.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = HashMap::from([
            (
                String::from("x"),
                Union::I1(Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                }),
            ),
            (
                String::from("a"),
                Union::I1(Signal {
                    id: String::from("a_1"),
                    scope: Scope::Memory,
                }),
            ),
        ]);
        assert_eq!(context_map, control)
    }

    #[test]
    fn should_add_called_node_id_to_the_identifier_creator_if_id_is_not_used() {
        let memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                String::from("my_node_o_y"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        let mut context_map = HashMap::from([(
            String::from("x"),
            Union::I1(Signal {
                id: String::from("a"),
                scope: Scope::Local,
            }),
        )]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        memory.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = IdentifierCreator::from(vec![String::from("a"), String::from("my_node_o_y")]);

        assert_eq!(identifier_creator, control)
    }

    #[test]
    fn should_add_called_node_signal_to_the_context_if_id_is_already_used() {
        let memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                String::from("a"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        let mut context_map = HashMap::from([(
            String::from("x"),
            Union::I1(Signal {
                id: String::from("a"),
                scope: Scope::Local,
            }),
        )]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        memory.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = HashMap::from([
            (
                String::from("x"),
                Union::I1(Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                }),
            ),
            (
                String::from("a"),
                Union::I1(Signal {
                    id: String::from("a_1"),
                    scope: Scope::Memory,
                }),
            ),
        ]);
        assert_eq!(context_map, control)
    }
}

#[cfg(test)]
mod replace_by_context {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::constant::Constant;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::memory::{Buffer, CalledNode, Memory};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_replace_all_occurence_of_identifiers_in_buffers_by_context() {
        let memory = Memory {
            buffers: HashMap::from([(
                format!("z"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::MapApplication {
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
                                    id: String::from("x"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                            },
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("y"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("x"), 0),
                            (String::from("y"), 0),
                        ]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };

        let context_map = HashMap::from([
            (
                String::from("x"),
                Union::I1(Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                }),
            ),
            (
                String::from("y"),
                Union::I2(StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("/2"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("b"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                }),
            ),
            (
                String::from("z"),
                Union::I1(Signal {
                    id: String::from("c"),
                    scope: Scope::Local,
                }),
            ),
        ]);

        let replaced_memory = memory.replace_by_context(&context_map);

        let control = Memory {
            buffers: HashMap::from([(
                format!("c"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::MapApplication {
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
                                    id: String::from("a"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("a"), 0)]),
                            },
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("/2"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("b"),
                                        scope: Scope::Local,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("a"), 0),
                            (String::from("b"), 0),
                        ]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };

        assert_eq!(replaced_memory, control)
    }

    #[test]
    fn should_replace_all_occurence_of_identifiers_in_called_nodes_by_context() {
        let memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_node_o_z"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        let context_map = HashMap::from([(
            String::from("my_node_o_z"),
            Union::I1(Signal {
                id: String::from("my_node_o_z1"),
                scope: Scope::Memory,
            }),
        )]);

        let replaced_memory = memory.replace_by_context(&context_map);

        let control = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_node_o_z1"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        assert_eq!(replaced_memory, control)
    }
}

#[cfg(test)]
mod remove_called_node {
    use std::collections::HashMap;

    use crate::hir::memory::{CalledNode, Memory};

    #[test]
    fn should_remove_called_node_from_memory_if_exist() {
        let mut memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_node_o_z"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        memory.remove_called_node(&format!("my_node_o_z"));

        let control = Memory::new();

        assert_eq!(memory, control)
    }

    #[test]
    fn should_do_nothing_if_called_node_is_not_in_memory() {
        let mut memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_node_o_z"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        memory.remove_called_node(&format!("other_node_o_z"));

        let control = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_node_o_z"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        assert_eq!(memory, control)
    }
}

#[cfg(test)]
mod combine {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::constant::Constant;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::memory::{Buffer, CalledNode, Memory};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_combine_called_nodes_when_they_differ() {
        let mut memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_node_o_z"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };
        let other = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("other_node_o_y"),
                CalledNode {
                    node_id: format!("other_node"),
                    signal_id: format!("o"),
                },
            )]),
        };

        memory.combine(other);

        let control = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([
                (
                    format!("my_node_o_z"),
                    CalledNode {
                        node_id: format!("my_node"),
                        signal_id: format!("o"),
                    },
                ),
                (
                    format!("other_node_o_y"),
                    CalledNode {
                        node_id: format!("other_node"),
                        signal_id: format!("o"),
                    },
                ),
            ]),
        };

        assert_eq!(memory, control)
    }

    #[test]
    fn should_combine_buffers_when_their_id_differ() {
        let mut memory = Memory {
            buffers: HashMap::from([(
                format!("c"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::MapApplication {
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
                                    id: String::from("a"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("a"), 0)]),
                            },
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("/2"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("b"),
                                        scope: Scope::Local,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("a"), 0),
                            (String::from("b"), 0),
                        ]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };
        let other = Memory {
            buffers: HashMap::from([(
                format!("z"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::MapApplication {
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
                                    id: String::from("x"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                            },
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("y"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("x"), 0),
                            (String::from("y"), 0),
                        ]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };
        memory.combine(other);

        let control = Memory {
            buffers: HashMap::from([
                (
                    format!("c"),
                    Buffer {
                        typing: Type::Integer,
                        initial_value: Constant::Integer(0),
                        expression: StreamExpression::MapApplication {
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
                                        id: String::from("a"),
                                        scope: Scope::Local,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("a"), 0)]),
                                },
                                StreamExpression::MapApplication {
                                    function_expression: Expression::Call {
                                        id: String::from("/2"),
                                        typing: Some(Type::Abstract(
                                            vec![Type::Integer],
                                            Box::new(Type::Integer),
                                        )),
                                        location: Location::default(),
                                    },
                                    inputs: vec![StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("b"),
                                            scope: Scope::Local,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("b"),
                                            0,
                                        )]),
                                    }],
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("a"), 0),
                                (String::from("b"), 0),
                            ]),
                        },
                    },
                ),
                (
                    format!("z"),
                    Buffer {
                        typing: Type::Integer,
                        initial_value: Constant::Integer(0),
                        expression: StreamExpression::MapApplication {
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
                                        id: String::from("x"),
                                        scope: Scope::Local,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Local,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
                        },
                    },
                ),
            ]),
            called_nodes: HashMap::new(),
        };

        assert_eq!(memory, control)
    }
}
