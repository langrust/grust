use std::collections::HashMap;

use crate::{
    common::graph::{color::Color, Graph},
    hir::{
        equation::Equation, identifier_creator::IdentifierCreator, memory::Memory, node::Node,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    },
};

use super::Union;

impl UnitaryNode {
    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// node fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed(&mut self, nodes: &HashMap<String, Node>) {
        // create identifier creator containing the signals
        let mut identifier_creator = IdentifierCreator::from(self.get_signals());
        // get graph
        let graph = self.graph.get_mut().unwrap();
        // compute new equations for the unitary node
        let mut new_equations: Vec<Equation> = vec![];
        self.equations.iter().for_each(|equation| {
            let mut retrieved_equations = equation.inline_when_needed_reccursive(
                &mut self.memory,
                &mut identifier_creator,
                graph,
                nodes,
            );
            new_equations.append(&mut retrieved_equations)
        });

        // update node's unitary node
        self.update_equations(&new_equations)
    }

    /// Instantiate unitary node's equations with inputs.
    ///
    /// It will return new equations where the input signals are instanciated by
    /// expressions.
    /// New equations should have fresh id according to the calling node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node to_be_inlined(i: int) {
    ///     o: int = 0 fby j;
    ///     out j: int = i + 1;
    /// }
    ///
    /// node calling_node(i: int) {
    ///     out o: int = to_be_inlined(o);
    ///     j: int = i * o;
    /// }
    /// ```
    ///
    /// The call to `to_be_inlined` will generate th following equations:
    ///
    /// ```GR
    /// o: int = 0 fby j_1;
    /// j_1: int = o + 1;
    /// ```
    pub fn instantiate_equations_and_memory(
        &self,
        identifier_creator: &mut IdentifierCreator,
        inputs: &[(String, StreamExpression)],
        new_output_signal: Option<Signal>,
    ) -> (Vec<Equation>, Memory) {
        // create the context with the given inputs
        let mut context_map = inputs
            .iter()
            .map(|(input, expression)| (input.clone(), Union::I2(expression.clone())))
            .collect::<HashMap<_, _>>();

        // add output to context
        let same_output = new_output_signal.map_or(false, |new_output_signal| {
            if self.output_id != new_output_signal.id {
                context_map.insert(self.output_id.clone(), Union::I1(new_output_signal));
                false
            } else {
                true
            }
        });

        // add identifiers of the inlined equations to the context
        self.equations.iter().for_each(|equation| {
            if !same_output || (equation.id != self.output_id) {
                equation.add_necessary_renaming(identifier_creator, &mut context_map)
            }
        });
        // add identifiers of the inlined memory to the context
        self.memory
            .add_necessary_renaming(identifier_creator, &mut context_map);

        // reduce equations according to the context
        let equations = self
            .equations
            .iter()
            .map(|equation| equation.replace_by_context(&context_map))
            .collect();

        // reduce memory according to the context
        let memory = self.memory.replace_by_context(&context_map);

        (equations, memory)
    }

    /// Update unitary node equations and add the corresponding dependency graph.
    pub fn update_equations(&mut self, new_equations: &[Equation]) {
        // put new equations in unitary node
        self.equations = new_equations.to_vec();
        // add a dependency graph to the unitary node
        let mut graph = Graph::new();
        self.get_signals()
            .iter()
            .for_each(|signal_id| graph.add_vertex(signal_id.clone(), Color::White));
        self.memory
            .buffers
            .keys()
            .for_each(|signal_id| graph.add_vertex(signal_id.clone(), Color::White));
        self.equations.iter().for_each(
            |Equation {
                 id: from,
                 expression,
                 ..
             }| {
                for (to, weight) in expression.get_dependencies() {
                    graph.add_weighted_edge(from, to.clone(), *weight)
                }
            },
        );
        self.graph = OnceCell::from(graph);
    }
}

#[cfg(test)]
mod instantiate_equations_and_memory {

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::memory::Memory;
    use crate::hir::unitary_node::UnitaryNode;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_instantiate_nodes_equations_with_the_given_inputs_without_output_infos() {
        // node calling_node(i: int) {
        //     o: int = to_be_inlined(o);
        //     out j: int = i * o;
        // }
        let mut identifier_creator = IdentifierCreator::from(vec![
            String::from("i"),
            String::from("j"),
            String::from("o"),
        ]);

        // node to_be_inlined(i: int) {
        //     out o: int = 0 fby j;
        //     j: int = i + 1;
        // }
        let unitary_node = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("to_be_inlined"),
            output_id: String::from("o"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("j"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Local,
                    id: String::from("j"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FunctionApplication {
                        function_expression: Expression::Identifier {
                            id: String::from("+1"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let (equations, _) = unitary_node.instantiate_equations_and_memory(
            &mut identifier_creator,
            &vec![(
                format!("i"),
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                },
            )],
            None,
        );

        // out o_1: int = 0 fby j_1;
        // j_1: int = o + 1;
        let control = vec![
            Equation {
                scope: Scope::Output,
                id: String::from("o_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("j_1"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j_1"), 0)]),
                    }),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("j_1"), 1)]),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("j_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::FunctionApplication {
                    function_expression: Expression::Identifier {
                        id: String::from("+1"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("o"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                },
                location: Location::default(),
            },
        ];

        assert_eq!(equations.len(), control.len());
        for equation in equations {
            assert!(control
                .iter()
                .any(|control_equation| &equation == control_equation))
        }
    }

    #[test]
    fn should_instantiate_nodes_equations_with_the_given_inputs_with_output_infos() {
        // node calling_node(i: int) {
        //     o: int = to_be_inlined(o);
        //     out j: int = i * o;
        // }
        let mut identifier_creator = IdentifierCreator::from(vec![
            String::from("i"),
            String::from("j"),
            String::from("o"),
        ]);

        // node to_be_inlined(i: int) {
        //     out o: int = 0 fby j;
        //     j: int = i + 1;
        // }
        let unitary_node = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("to_be_inlined"),
            output_id: String::from("o"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("j"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Local,
                    id: String::from("j"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FunctionApplication {
                        function_expression: Expression::Identifier {
                            id: String::from("+1"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let (equations, _) = unitary_node.instantiate_equations_and_memory(
            &mut identifier_creator,
            &vec![(
                format!("i"),
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                },
            )],
            Some(Signal {
                id: String::from("o"),
                scope: Scope::Local,
            }),
        );

        // out o: int = 0 fby j_1;
        // j_1: int = o + 1;
        let control = vec![
            Equation {
                scope: Scope::Output,
                id: String::from("o"),
                signal_type: Type::Integer,
                expression: StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("j_1"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j_1"), 0)]),
                    }),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("j_1"), 1)]),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("j_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::FunctionApplication {
                    function_expression: Expression::Identifier {
                        id: String::from("+1"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("o"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                },
                location: Location::default(),
            },
        ];

        assert_eq!(equations.len(), control.len());
        for equation in equations {
            assert!(control
                .iter()
                .any(|control_equation| &equation == control_equation))
        }
    }

    #[test]
    fn should_instantiate_nodes_memory_with_the_given_inputs_without_output_infos() {
        // node calling_node(i: int) {
        //     o: int = to_be_inlined(o);
        //     out j: int = i * o;
        // }
        let mut identifier_creator = IdentifierCreator::from(vec![
            String::from("i"),
            String::from("j"),
            String::from("o"),
        ]);

        // node to_be_inlined(i: int) {
        //     out o: int = mem_o;
        // }
        // memory { mem_o: i + 1}
        let mut memory = Memory::new();
        memory.add_buffer(
            format!("mem_o"),
            Constant::Integer(0),
            StreamExpression::FunctionApplication {
                function_expression: Expression::Identifier {
                    id: String::from("+1"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("i"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
            },
        );
        let unitary_node = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("to_be_inlined"),
            output_id: String::from("o"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o"),
                signal_type: Type::Integer,
                expression: StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("mem_o"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("mem_o"), 0)]),
                },
                location: Location::default(),
            }],
            memory,
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let (_, memory) = unitary_node.instantiate_equations_and_memory(
            &mut identifier_creator,
            &vec![(
                format!("i"),
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                },
            )],
            None,
        );

        // memory { mem_o: o + 1}
        let mut control = Memory::new();
        control.add_buffer(
            format!("mem_o"),
            Constant::Integer(0),
            StreamExpression::FunctionApplication {
                function_expression: Expression::Identifier {
                    id: String::from("+1"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
            },
        );

        assert_eq!(memory, control)
    }

    #[test]
    fn should_instantiate_nodes_memory_with_the_given_inputs_with_output_infos() {
        // node calling_node(i: int) {
        //     o: int = to_be_inlined(o);
        //     out j: int = i * o;
        // }
        let mut identifier_creator = IdentifierCreator::from(vec![
            String::from("i"),
            String::from("j"),
            String::from("o"),
        ]);

        // node to_be_inlined(i: int) {
        //     out o: int = mem_o;
        // }
        // memory { mem_o: i + 1}
        let mut memory = Memory::new();
        memory.add_buffer(
            format!("mem_o"),
            Constant::Integer(0),
            StreamExpression::FunctionApplication {
                function_expression: Expression::Identifier {
                    id: String::from("+1"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("i"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
            },
        );
        let unitary_node = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("to_be_inlined"),
            output_id: String::from("o"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o"),
                signal_type: Type::Integer,
                expression: StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("mem_o"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("mem_o"), 0)]),
                },
                location: Location::default(),
            }],
            memory,
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let (_, memory) = unitary_node.instantiate_equations_and_memory(
            &mut identifier_creator,
            &vec![(
                format!("i"),
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                },
            )],
            Some(Signal {
                id: String::from("o"),
                scope: Scope::Local,
            }),
        );

        // memory { mem_o: o + 1}
        let mut control = Memory::new();
        control.add_buffer(
            format!("mem_o"),
            Constant::Integer(0),
            StreamExpression::FunctionApplication {
                function_expression: Expression::Identifier {
                    id: String::from("+1"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
            },
        );

        assert_eq!(memory, control)
    }
}
