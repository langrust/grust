use std::collections::HashMap;

use crate::common::{location::Location, type_system::Type};
use crate::hir::{equation::Equation, identifier_creator::IdentifierCreator, memory::Memory};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust unitary node HIR.
pub struct UnitaryNode {
    /// Mother node identifier.
    pub node_id: String,
    /// Output signal identifier.
    pub output_id: String,
    /// Unitary node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Unitary node's scheduled equations.
    pub scheduled_equations: Vec<Equation>,
    /// Unitary node's memory.
    pub memory: Memory,
    /// Mother node location.
    pub location: Location,
}

impl UnitaryNode {
    /// Normalize HIR unitary nodes.
    ///
    /// Normalize HIR unitary node's equations as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    /// use grustine::hir::{
    ///     equation::Equation, expression::Expression, memory::Memory,
    ///     stream_expression::StreamExpression, unitary_node::UnitaryNode,
    /// };
    ///
    /// let unitary_nodes_used_inputs = HashMap::from([(
    ///     String::from("my_node"),
    ///     HashMap::from([(String::from("o"), vec![true, true])]),
    /// )]);
    ///
    /// let equation = Equation {
    ///     scope: Scope::Output,
    ///     id: String::from("x"),
    ///     signal_type: Type::Integer,
    ///     expression: StreamExpression::MapApplication {
    ///         function_expression: Expression::Call {
    ///             id: String::from("+"),
    ///             typing: Type::Abstract(
    ///                 vec![Type::Integer, Type::Integer],
    ///                 Box::new(Type::Integer)
    ///             ),
    ///             location: Location::default(),
    ///         },
    ///         inputs: vec![
    ///             StreamExpression::Constant {
    ///                 constant: Constant::Integer(1),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             StreamExpression::NodeApplication {
    ///                 node: String::from("my_node"),
    ///                 inputs: vec![
    ///                     StreamExpression::SignalCall {
    ///                         id: String::from("s"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     },
    ///                     StreamExpression::MapApplication {
    ///                         function_expression: Expression::Call {
    ///                             id: String::from("*2"),
    ///                             typing: Type::Abstract(
    ///                                 vec![Type::Integer],
    ///                                 Box::new(Type::Integer),
    ///                             ),
    ///                             location: Location::default(),
    ///                         },
    ///                         inputs: vec![StreamExpression::SignalCall {
    ///                             id: String::from("v"),
    ///                             typing: Type::Integer,
    ///                             location: Location::default(),
    ///                         }],
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     },
    ///                 ],
    ///                 signal: String::from("o"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///         ],
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///     },
    ///     location: Location::default(),
    /// };
    /// let mut unitary_node = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: vec![equation],
    ///     memory: Memory::new(),
    ///     location: Location::default(),
    /// };
    /// unitary_node.normalize(&unitary_nodes_used_inputs);
    ///
    /// let equations = vec![
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_1"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("*2"),
    ///                 typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![StreamExpression::SignalCall {
    ///                 id: String::from("v"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_2"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::UnitaryNodeApplication {
    ///             node: String::from("my_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("s"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Output,
    ///         id: String::from("x"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("+"),
    ///                 typing: Type::Abstract(
    ///                     vec![Type::Integer, Type::Integer],
    ///                     Box::new(Type::Integer)
    ///                 ),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![
    ///                 StreamExpression::Constant {
    ///                     constant: Constant::Integer(1),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_2"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     }
    /// ];
    /// let control = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("x"),
    ///     inputs: vec![(String::from("s"), Type::Integer), (String::from("v"), Type::Integer)],
    ///     scheduled_equations: equations,
    ///     memory: Memory::new(),
    ///     location: Location::default(),
    /// };
    /// assert_eq!(unitary_node, control);
    /// ```
    pub fn normalize(
        &mut self,
        unitary_nodes_used_inputs: &HashMap<String, HashMap<String, Vec<bool>>>,
    ) {
        let mut identifier_creator = IdentifierCreator::new(self);

        let UnitaryNode {
            scheduled_equations,
            ..
        } = self;

        *scheduled_equations = scheduled_equations
            .clone()
            .into_iter()
            .flat_map(|equation| {
                equation.normalize(&mut identifier_creator, unitary_nodes_used_inputs)
            })
            .collect();
    }

    /// Create memory for HIR unitary nodes.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         memmy_nodeo: (my_node, o);
    ///     },
    /// }
    /// ```
    ///
    /// This example is tested in source.
    pub fn memorize(&mut self) {
        let mut identifier_creator = IdentifierCreator::new(self);
        let mut memory = Memory::new();

        self.scheduled_equations
            .iter_mut()
            .for_each(|equation| equation.memorize(&mut identifier_creator, &mut memory));

        self.memory = memory;
    }
}

#[cfg(test)]
mod memorize {
    use crate::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    use crate::hir::{
        equation::Equation, expression::Expression, memory::Memory,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };

    #[test]
    fn should_memorize_followed_by() {
        let equation = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    ),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: String::from("s"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("v"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let mut unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            scheduled_equations: vec![equation],
            memory: Memory::new(),
            location: Location::default(),
        };
        unitary_node.memorize();

        let equation = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    ),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: String::from("s"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("mem"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let mut memory = Memory::new();
        memory.add_buffer(
            String::from("mem"),
            Constant::Integer(0),
            StreamExpression::SignalCall {
                id: String::from("v"),
                typing: Type::Integer,
                location: Location::default(),
            },
        );
        let control = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            scheduled_equations: vec![equation],
            memory,
            location: Location::default(),
        };
        assert_eq!(unitary_node, control);
    }

    #[test]
    fn should_memorize_node_expression() {
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("*2"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("x_2"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("s"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Output,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("+"),
                        typing: Type::Abstract(
                            vec![Type::Integer, Type::Integer],
                            Box::new(Type::Integer),
                        ),
                        location: Location::default(),
                    },
                    inputs: vec![
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_2"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
        ];
        let mut unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            scheduled_equations: equations.clone(),
            memory: Memory::new(),
            location: Location::default(),
        };
        unitary_node.memorize();

        let mut memory = Memory::new();
        memory.add_called_node(
            String::from("memmy_nodeo"),
            String::from("my_node"),
            String::from("o"),
        );
        let control = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            scheduled_equations: equations,
            memory,
            location: Location::default(),
        };
        assert_eq!(unitary_node, control);
    }
}
