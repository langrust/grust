use std::collections::HashMap;

use crate::hir::{
    equation::Equation, identifier_creator::IdentifierCreator, signal::Signal,
    stream_expression::StreamExpression, unitary_node::UnitaryNode,
};

use super::Union;

impl UnitaryNode {
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
    pub fn instantiate_equations(
        &self,
        identifier_creator: &mut IdentifierCreator,
        inputs: &Vec<(String, StreamExpression)>,
        new_output_signal: Option<Signal>,
    ) -> Vec<Equation> {
        // create the context with the given inputs
        let mut context_map = inputs
            .iter()
            .map(|(input, expression)| (input.clone(), Union::I2(expression.clone())))
            .collect::<HashMap<_, _>>();

        // add output to context
        let same_output = new_output_signal
            .clone()
            .map_or(false, |new_output_signal| {
                if self.output_id != new_output_signal.id {
                    context_map
                        .insert(self.output_id.clone(), Union::I1(new_output_signal.clone()));
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

        // reduce equations according to the context
        self.equations
            .iter()
            .map(|equation| equation.replace_by_context(&context_map))
            .collect()
    }
}

#[cfg(test)]
mod instantiate_equations {
    use once_cell::sync::OnceCell;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::memory::Memory;
    use crate::hir::unitary_node::UnitaryNode;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        signal::Signal, stream_expression::StreamExpression,
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
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
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

        let equations = unitary_node.instantiate_equations(
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
                            id: String::from("1"),
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
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
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
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
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

        let equations = unitary_node.instantiate_equations(
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
            Some(&String::from("o")),
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
                            id: String::from("1"),
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
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
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
}
