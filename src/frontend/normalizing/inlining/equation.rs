use std::collections::HashMap;

use crate::{
    common::{
        graph::{color::Color, Graph},
        scope::Scope,
    },
    hir::{
        equation::Equation, identifier_creator::IdentifierCreator, node::Node,
        stream_expression::StreamExpression,
    },
};

use super::Union;

impl Equation {
    /// Add the equation identifier to the identifier creator.
    ///
    /// It will add the equation identifier to the identifier creator.
    /// If the identifier already exists, then the new identifer created by
    /// the identifier creator will be added to the renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<String, Union<String, StreamExpression>>,
    ) {
        let new_id =
            identifier_creator.new_identifier(String::new(), self.id.clone(), String::new());
        if new_id.ne(&self.id) {
            assert!(context_map
                .insert(self.id.clone(), Union::I1(new_id))
                .is_none());
        }
    }

    /// Replace identifier occurence by element in context.
    ///
    /// It will return a new equation where the expression has been modified
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
        context_map: &HashMap<String, Union<String, StreamExpression>>,
    ) -> Equation {
        let mut new_equation = self.clone();
        if let Some(element) = context_map.get(&new_equation.id) {
            match element {
                Union::I1(new_id) | Union::I2(StreamExpression::SignalCall { id: new_id, .. }) => {
                    new_equation.id = new_id.clone()
                }
                Union::I2(_) => unreachable!(),
            }
        }
        new_equation.expression.replace_by_context(context_map);
        new_equation
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// ```
    /// In this example, an equation `fib: int = semi_fib(fib).o` calls
    /// `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed(
        &self,
        identifier_creator: &mut IdentifierCreator,
        graph: &mut Graph<Color>,
        nodes: &HashMap<String, &Node>,
    ) -> Vec<Equation> {
        match &self.expression {
            StreamExpression::UnitaryNodeApplication {
                node,
                inputs,
                signal,
                ..
            } => {
                let mut inputs = inputs.clone();

                // inline potential node calls in the inputs
                let mut new_equations = inputs
                    .iter_mut()
                    .map(|expression| {
                        expression.inline_when_needed(&self.id, identifier_creator, graph, nodes)
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                // a loop in the graph induces that inputs depends on output
                let should_inline = graph.is_loop(&self.id);

                // then node call must be inlined
                if should_inline {
                    let called_node = nodes.get(node).unwrap();
                    let called_unitary_node = called_node.unitary_nodes.get(signal).unwrap();

                    // get equations from called node, with corresponding inputs
                    let mut retrieved_equations = called_unitary_node.instantiate_equations(
                        identifier_creator,
                        &inputs,
                        Some(&self.id),
                    );

                    retrieved_equations.iter_mut().for_each(|equation| {
                        if equation.scope == Scope::Output {
                            equation.scope = self.scope.clone()
                        }
                    });

                    new_equations.append(&mut retrieved_equations);
                }

                new_equations
            }
            _ => {
                let mut equation = self.clone();
                let mut new_equations = equation.expression.inline_when_needed(
                    &self.id,
                    identifier_creator,
                    graph,
                    nodes,
                );
                new_equations.push(equation);
                new_equations
            }
        }
    }
}

#[cfg(test)]
mod add_necessary_renaming {
    use std::collections::HashMap;

    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::identifier_creator::IdentifierCreator;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_the_equation_id_to_the_identifier_creator_if_id_is_not_used() {
        let equation = Equation {
            id: String::from("y"),
            scope: Scope::Local,
            expression: StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };

        let mut context_map = HashMap::from([(String::from("x"), Union::I1(String::from("a")))]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        equation.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = IdentifierCreator::from(vec![String::from("a"), String::from("y")]);

        assert_eq!(identifier_creator, control)
    }

    #[test]
    fn should_add_the_equation_id_to_the_context_if_id_is_already_used() {
        let equation = Equation {
            id: String::from("a"),
            scope: Scope::Local,
            expression: StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };

        let mut context_map = HashMap::from([(String::from("x"), Union::I1(String::from("a")))]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        equation.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = HashMap::from([
            (String::from("x"), Union::I1(String::from("a"))),
            (String::from("a"), Union::I1(String::from("a_1"))),
        ]);
        assert_eq!(context_map, control)
    }
}

#[cfg(test)]
mod replace_by_context {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, stream_expression::StreamExpression,
    };

    #[test]
    fn should_replace_all_occurence_of_identifiers_by_context() {
        let equation = Equation {
            id: String::from("z"),
            scope: Scope::Local,
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
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("y"),
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
            signal_type: Type::Integer,
            location: Location::default(),
        };

        let context_map = HashMap::from([
            (String::from("x"), Union::I1(String::from("a"))),
            (
                String::from("y"),
                Union::I2(StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("/2"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("b"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                }),
            ),
            (String::from("z"), Union::I1(String::from("c"))),
        ]);

        let replaced_equation = equation.replace_by_context(&context_map);

        let control = Equation {
            id: String::from("c"),
            scope: Scope::Local,
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
                        id: String::from("a"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            id: String::from("b"),
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
                    (String::from("x"), 0),
                    (String::from("y"), 0),
                ]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };

        assert_eq!(replaced_equation, control)
    }
}

#[cfg(test)]
mod inline_when_needed_visit {
    use once_cell::sync::OnceCell;

    use crate::ast::expression::Expression;
    use crate::common::graph::color::Color;
    use crate::common::graph::Graph;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::identifier_creator::IdentifierCreator;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, node::Node,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_remove_inline_root_node_calls_when_inputs_depends_on_outputs() {
        let mut nodes = HashMap::new();

        // node my_node(i: int, j: int) {
        //     out o: int = i + (0 fby j);
        // }
        let my_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
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
                        id: String::from("i"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("j"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        };
        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("i"), Type::Integer),
                (String::from("j"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("o"), my_node_equation.clone())]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("i"), Type::Integer),
                        (String::from("j"), Type::Integer),
                    ],
                    equations: vec![my_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_vertex(String::from("j"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("j"), 1);
        my_node.graph.set(graph).unwrap();
        nodes.insert(String::from("my_node"), &my_node);

        // node other_node(i: int) {
        //     out o: int = 0 fby i;
        // }
        let other_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    id: String::from("i"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
            },
            location: Location::default(),
        };
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                other_node_equation.clone(),
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("other_node"),
                    output_id: String::from("o"),
                    inputs: vec![(String::from("i"), Type::Integer)],
                    equations: vec![other_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 1);
        other_node.graph.set(graph).unwrap();
        nodes.insert(String::from("other_node"), &other_node);

        // x: int = my_node(v*2, x).o
        let equation_1 = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                node: String::from("my_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*2"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("v"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                ],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                node: String::from("other_node"),
                inputs: vec![StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                }],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        // node test(v: int) {
        //     x: int = my_node(v*2, x).o
        //     out y: int = other_node(x-1).o
        // }
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode {
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![equation_1.clone(), equation_2.clone()],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("x"), 1);
        graph.add_edge(&String::from("y"), String::from("x"), 1);
        node.graph.set(graph.clone()).unwrap();
        nodes.insert(String::from("test"), &node);

        let mut identifier_creator = IdentifierCreator::from(
            node.unitary_nodes
                .get(&String::from("y"))
                .unwrap()
                .get_signals(),
        );
        let new_equations =
            equation_1.inline_when_needed(&mut identifier_creator, &mut graph, &nodes);

        // x: int = v*2 + 0 fby x
        let control = [Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
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
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*2"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("v"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        }];

        assert_eq!(new_equations, control)
    }

    #[test]
    fn should_remove_inline_all_node_calls_when_inputs_depends_on_outputs() {
        let mut nodes = HashMap::new();

        // node my_node(i: int, j: int) {
        //     out o: int = i + (0 fby j);
        // }
        let my_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
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
                        id: String::from("i"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("j"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        };
        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("i"), Type::Integer),
                (String::from("j"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("o"), my_node_equation.clone())]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("i"), Type::Integer),
                        (String::from("j"), Type::Integer),
                    ],
                    equations: vec![my_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_vertex(String::from("j"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("j"), 1);
        my_node.graph.set(graph).unwrap();
        nodes.insert(String::from("my_node"), &my_node);

        // node other_node(i: int) {
        //     out o: int = 0 fby i;
        // }
        let other_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    id: String::from("i"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
            },
            location: Location::default(),
        };
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                other_node_equation.clone(),
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("other_node"),
                    output_id: String::from("o"),
                    inputs: vec![(String::from("i"), Type::Integer)],
                    equations: vec![other_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 1);
        other_node.graph.set(graph).unwrap();
        nodes.insert(String::from("other_node"), &other_node);

        // x: int = 1 + my_node(v*2, x).o
        let equation_1 = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("1+"),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::UnitaryNodeApplication {
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*2"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("v"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("v"), 0),
                        (String::from("x"), 1),
                    ]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                node: String::from("other_node"),
                inputs: vec![StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                }],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        // node test(v: int) {
        //     x: int = my_node(v*2, x).o
        //     out y: int = other_node(x-1).o
        // }
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode {
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![equation_1.clone(), equation_2.clone()],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("x"), 1);
        graph.add_edge(&String::from("y"), String::from("x"), 1);
        node.graph.set(graph.clone()).unwrap();
        nodes.insert(String::from("test"), &node);

        let mut identifier_creator = IdentifierCreator::from(
            node.unitary_nodes
                .get(&String::from("y"))
                .unwrap()
                .get_signals(),
        );
        let new_equations =
            equation_1.inline_when_needed(&mut identifier_creator, &mut graph, &nodes);

        // o: int = v*2 + 0 fby x
        let added_equation = Equation {
            scope: Scope::Local,
            id: String::from("o"),
            signal_type: Type::Integer,
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
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*2"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("v"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        };
        // x: int = 1 + o
        let inlined_equation = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("1+"),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("v"), 0),
                        (String::from("x"), 1),
                    ]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        let control = vec![added_equation, inlined_equation];

        assert_eq!(new_equations, control)
    }
}
