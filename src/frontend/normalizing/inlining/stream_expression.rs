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
            StreamExpression::NodeApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::UnitaryNodeApplication { .. } => unreachable!(),
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
