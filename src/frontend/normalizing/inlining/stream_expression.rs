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

impl StreamExpression {
    /// Replace identifier occurence by element in context.
    ///
    /// It will modify the expression according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` will become
    /// `a + b/2`.
    pub fn replace_by_context(
        &mut self,
        context_map: &HashMap<String, Union<String, StreamExpression>>,
    ) {
        match self {
            StreamExpression::Constant { .. } => (),
            StreamExpression::SignalCall { ref mut id, .. } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Union::I1(new_id) => *id = new_id.clone(),
                        Union::I2(new_expression) => *self = new_expression.clone(),
                    }
                }
            }
            StreamExpression::FollowedBy { expression, .. } => {
                expression.replace_by_context(context_map)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::UnitaryNodeApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .for_each(|(_, expression)| expression.replace_by_context(context_map)),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                expression.replace_by_context(context_map);
                arms.iter_mut().for_each(|(_, bound, body, expression)| {
                    // todo!("get pattern's context");
                    bound
                        .as_mut()
                        .map(|expression| expression.replace_by_context(context_map));
                    body.iter_mut()
                        .for_each(|equation| equation.expression.replace_by_context(context_map));
                    expression.replace_by_context(context_map);
                })
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                option.replace_by_context(context_map);
                present_body
                    .iter_mut()
                    .for_each(|equation| equation.expression.replace_by_context(context_map));
                present.replace_by_context(context_map);
                default_body
                    .iter_mut()
                    .for_each(|equation| equation.expression.replace_by_context(context_map));
                default.replace_by_context(context_map);
            }
        }
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
    /// In this example, if an expression `semi_fib(fib).o` is assigned to the
    /// signal `fib` there is no causality loop.
    /// But we need to inline the code, a function can not compute an output
    /// before knowing the input.
    pub fn inline_when_needed(
        &mut self,
        signal_id: &String,
        identifier_creator: &mut IdentifierCreator,
        graph: &mut Graph<Color>,
        nodes: &HashMap<String, &Node>,
    ) -> Vec<Equation> {
        match self {
            StreamExpression::UnitaryNodeApplication {
                node,
                inputs,
                signal,
                typing,
                location,
                dependencies,
            } => {
                // inline potential node calls in the inputs
                let mut new_equations = inputs
                    .iter_mut()
                    .map(|expression| {
                        expression.inline_when_needed(signal_id, identifier_creator, graph, nodes)
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                // a loop in the graph induces that inputs depends on output
                let should_inline = graph.is_loop(signal_id);

                // then node call must be inlined
                if should_inline {
                    let called_node = nodes.get(node).unwrap();
                    let called_unitary_node = called_node.unitary_nodes.get(signal).unwrap();

                    // get equations from called node, with corresponding inputs
                    let mut retrieved_equations =
                        called_unitary_node.instantiate_equations(identifier_creator, inputs, None);

                    // change the expression to a signal call to the output signal
                    let typing = typing.clone();
                    let location = location.clone();
                    let dependencies = dependencies.clone();
                    retrieved_equations.iter_mut().for_each(|equation| {
                        if equation.scope == Scope::Output {
                            *self = StreamExpression::SignalCall {
                                id: equation.id.clone(),
                                typing: typing.clone(),
                                location: location.clone(),
                                dependencies: dependencies.clone(),
                            };
                            equation.scope = Scope::Local
                        }
                    });

                    new_equations.append(&mut retrieved_equations);
                }

                new_equations
            }
            StreamExpression::Constant { .. } => vec![],
            StreamExpression::SignalCall { .. } => vec![],
            StreamExpression::FollowedBy { expression, .. } => {
                expression.inline_when_needed(signal_id, identifier_creator, graph, nodes)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .map(|expression| {
                    expression.inline_when_needed(signal_id, identifier_creator, graph, nodes)
                })
                .flatten()
                .collect(),
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .map(|(_, expression)| {
                    expression.inline_when_needed(signal_id, identifier_creator, graph, nodes)
                })
                .flatten()
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .map(|expression| {
                    expression.inline_when_needed(signal_id, identifier_creator, graph, nodes)
                })
                .flatten()
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut new_equations =
                    expression.inline_when_needed(signal_id, identifier_creator, graph, nodes);

                let mut other_new_equations = arms
                    .iter_mut()
                    .map(|(_, bound, body, expression)| {
                        assert!(body.is_empty());
                        let mut new_equations_bound = bound
                            .as_mut()
                            .map(|expression| {
                                expression.inline_when_needed(
                                    signal_id,
                                    identifier_creator,
                                    graph,
                                    nodes,
                                )
                            })
                            .unwrap_or_default();
                        let mut new_equations_expression = expression.inline_when_needed(
                            signal_id,
                            identifier_creator,
                            graph,
                            nodes,
                        );
                        new_equations_bound.append(&mut new_equations_expression);
                        new_equations_bound
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                new_equations.append(&mut other_new_equations);

                new_equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                assert!(present_body.is_empty());
                assert!(default_body.is_empty());

                let mut new_equations_option =
                    option.inline_when_needed(signal_id, identifier_creator, graph, nodes);
                let mut new_equations_present =
                    present.inline_when_needed(signal_id, identifier_creator, graph, nodes);
                let mut new_equations_default =
                    default.inline_when_needed(signal_id, identifier_creator, graph, nodes);
                new_equations_option.append(&mut new_equations_present);
                new_equations_option.append(&mut new_equations_default);

                new_equations_option
            }
        }
    }
}

#[cfg(test)]
mod replace_by_context {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::{dependencies::Dependencies, stream_expression::StreamExpression};

    #[test]
    fn should_replace_all_occurence_of_identifiers_by_context() {
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
            dependencies: Dependencies::from(vec![(String::from("x"), 0), (String::from("y"), 0)]),
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
        ]);

        expression.replace_by_context(&context_map);

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
                    id: String::from("a"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                },
                StreamExpression::MapApplication {
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
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0), (String::from("y"), 0)]),
        };

        assert_eq!(expression, control)
    }
}

#[cfg(test)]
mod inline_when_needed {
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
    fn should_inline_node_calls_when_inputs_depends_on_outputs() {
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
        let mut expression_1 = StreamExpression::MapApplication {
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
            dependencies: Dependencies::from(vec![(String::from("v"), 0), (String::from("x"), 1)]),
        };
        let equation_1 = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: expression_1.clone(),
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
        let new_equations = expression_1.inline_when_needed(
            &String::from("x"),
            &mut identifier_creator,
            &mut graph,
            &nodes,
        );

        // o: int = v*2 + 0 fby x
        let control = vec![Equation {
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
        }];
        assert_eq!(new_equations, control);
        // x: int = 1 + o
        let control = StreamExpression::MapApplication {
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
            dependencies: Dependencies::from(vec![(String::from("v"), 0), (String::from("x"), 1)]),
        };
        assert_eq!(expression_1, control);
    }
}
