use crate::ast::{expression::Expression, pattern::Pattern};
use crate::common::scope::Scope;
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{
    dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
    memory::Memory, signal::Signal,
};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression HIR.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Signal call stream expression.
    SignalCall {
        /// Signal identifier.
        id: String,
        /// Signal scope.
        scope: Scope,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Constant,
        /// The buffered expression.
        expression: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Map application stream expression.
    MapApplication {
        /// The expression applied.
        function_expression: Expression,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Node application stream expression.
    NodeApplication {
        /// The node applied.
        node: String,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// The signal retrieved.
        signal: String,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Unitary node application stream expression.
    UnitaryNodeApplication {
        /// The node state identifier.
        id: Option<String>,
        /// The mother node type.
        node: String,
        /// The output signal corresponding to the unitary node.
        signal: String,
        /// The inputs to the expression.
        inputs: Vec<(String, StreamExpression)>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Structure stream expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, StreamExpression)>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Array stream expression.
    Array {
        /// The elements inside the array.
        elements: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Pattern matching stream expression.
    Match {
        /// The stream expression to match.
        expression: Box<StreamExpression>,
        /// The different matching cases.
        arms: Vec<(
            Pattern,
            Option<StreamExpression>,
            Vec<Equation>,
            StreamExpression,
        )>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// When present stream expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional stream expression.
        option: Box<StreamExpression>,
        /// The body of present case when normalized.
        present_body: Vec<Equation>,
        /// The stream expression when present.
        present: Box<StreamExpression>,
        /// The body of default case when normalized.
        default_body: Vec<Equation>,
        /// The default stream expression.
        default: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
}

impl StreamExpression {
    /// Get the reference to the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::new(),
    /// };
    /// let typing = stream_expression.get_type();
    /// assert_eq!(typing, &Type::Integer)
    /// ```
    pub fn get_type(&self) -> &Type {
        match self {
            StreamExpression::Constant { typing, .. }
            | StreamExpression::SignalCall { typing, .. }
            | StreamExpression::FollowedBy { typing, .. }
            | StreamExpression::MapApplication { typing, .. }
            | StreamExpression::NodeApplication { typing, .. }
            | StreamExpression::UnitaryNodeApplication { typing, .. }
            | StreamExpression::Structure { typing, .. }
            | StreamExpression::Array { typing, .. }
            | StreamExpression::Match { typing, .. }
            | StreamExpression::When { typing, .. } => typing,
        }
    }

    /// Get the reference to the stream expression's location.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::new(),
    /// };
    /// let location = stream_expression.get_location();
    /// assert_eq!(location, &Location::default())
    /// ```
    pub fn get_location(&self) -> &Location {
        match self {
            StreamExpression::Constant { location, .. }
            | StreamExpression::SignalCall { location, .. }
            | StreamExpression::FollowedBy { location, .. }
            | StreamExpression::MapApplication { location, .. }
            | StreamExpression::NodeApplication { location, .. }
            | StreamExpression::UnitaryNodeApplication { location, .. }
            | StreamExpression::Structure { location, .. }
            | StreamExpression::Array { location, .. }
            | StreamExpression::Match { location, .. }
            | StreamExpression::When { location, .. } => location,
        }
    }

    /// Get the reference to the stream expression's dependencies.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::from(vec![]),
    /// };
    /// let dependencies = stream_expression.get_dependencies();
    /// assert_eq!(*dependencies, vec![])
    /// ```
    pub fn get_dependencies(&self) -> &Vec<(String, usize)> {
        match self {
            StreamExpression::Constant { dependencies, .. }
            | StreamExpression::SignalCall { dependencies, .. }
            | StreamExpression::FollowedBy { dependencies, .. }
            | StreamExpression::MapApplication { dependencies, .. }
            | StreamExpression::NodeApplication { dependencies, .. }
            | StreamExpression::UnitaryNodeApplication { dependencies, .. }
            | StreamExpression::Structure { dependencies, .. }
            | StreamExpression::Array { dependencies, .. }
            | StreamExpression::Match { dependencies, .. }
            | StreamExpression::When { dependencies, .. } => dependencies.get().unwrap(),
        }
    }

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
    pub fn memorize(&mut self, identifier_creator: &mut IdentifierCreator, memory: &mut Memory) {
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
                    String::from(""),
                    String::from(""),
                );
                memory.add_buffer(memory_id.clone(), constant.clone(), *expression.clone());
                // todo : this should be a memory call
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
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.memorize(identifier_creator, memory)),
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication {
                id, node, signal, ..
            } => memory.add_called_node(id.clone().unwrap(), node.clone(), signal.clone()),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .for_each(|(_, expression)| expression.memorize(identifier_creator, memory)),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .for_each(|expression| expression.memorize(identifier_creator, memory)),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                expression.memorize(identifier_creator, memory);
                arms.iter_mut()
                    .for_each(|(_, bound_expression, equations, expression)| {
                        bound_expression
                            .as_mut()
                            .map(|expression| expression.memorize(identifier_creator, memory));
                        equations
                            .iter_mut()
                            .for_each(|equation| equation.memorize(identifier_creator, memory));
                        expression.memorize(identifier_creator, memory)
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
                option.memorize(identifier_creator, memory);
                present_body
                    .iter_mut()
                    .for_each(|equation| equation.memorize(identifier_creator, memory));
                present.memorize(identifier_creator, memory);
                default_body
                    .iter_mut()
                    .for_each(|equation| equation.memorize(identifier_creator, memory));
                default.memorize(identifier_creator, memory)
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
        expression.memorize(&mut identifier_creator, &mut memory);

        let mut control = Memory::new();
        control.add_buffer(
            String::from("mem"),
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
                        id: String::from("m"),
                        scope: Scope::Memory,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("mem"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 1)]),
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
                            id: String::from("1"),
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
        expression.memorize(&mut identifier_creator, &mut memory);

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
                            id: String::from("1"),
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
