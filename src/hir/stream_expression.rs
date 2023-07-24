use std::collections::HashMap;

use crate::common::scope::Scope;
use crate::common::{constant::Constant, location::Location, pattern::Pattern, r#type::Type};
use crate::hir::{
    dependencies::Dependencies, equation::Equation, expression::Expression,
    identifier_creator::IdentifierCreator, memory::Memory,
};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression AST.
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
        /// The mother node.
        node: String,
        /// The output signal corresponding to the unitary node.
        signal: String,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
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

    /// Normalize HIR expressions.
    ///
    /// Normalize HIR expressions as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use std::collections::{HashSet, HashMap};
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, r#type::Type};
    /// use grustine::hir::{
    ///     dependencies::Dependencies, equation::Equation, expression::Expression,
    ///     identifier_creator::IdentifierCreator, stream_expression::StreamExpression,
    /// };
    ///
    /// let mut identifier_creator = IdentifierCreator {
    ///     signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
    /// };
    ///
    /// let mut expression = StreamExpression::MapApplication {
    ///     function_expression: Expression::Call {
    ///         id: String::from("+"),
    ///         typing: Type::Abstract(
    ///             vec![Type::Integer, Type::Integer],
    ///             Box::new(Type::Integer)
    ///         ),
    ///         location: Location::default(),
    ///     },
    ///     inputs: vec![
    ///         StreamExpression::Constant {
    ///             constant: Constant::Integer(1),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![]),
    ///         },
    ///         StreamExpression::UnitaryNodeApplication {
    ///             node: String::from("my_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("s"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
    ///                 },
    ///                 StreamExpression::MapApplication {
    ///                     function_expression: Expression::Call {
    ///                         id: String::from("*2"),
    ///                         typing: Type::Abstract(
    ///                             vec![Type::Integer],
    ///                             Box::new(Type::Integer),
    ///                         ),
    ///                         location: Location::default(),
    ///                     },
    ///                     inputs: vec![StreamExpression::SignalCall {
    ///                         id: String::from("v"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                         dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
    ///                     }],
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
    ///         },
    ///     ],
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
    /// };
    /// let equations = expression.normalize(&mut identifier_creator);
    ///
    /// let control = vec![
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
    ///                 dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
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
    ///                     dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                     dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
    ///         },
    ///         location: Location::default(),
    ///     },
    /// ];
    /// assert_eq!(equations, control);
    ///
    /// let control = StreamExpression::MapApplication {
    ///     function_expression: Expression::Call {
    ///         id: String::from("+"),
    ///         typing: Type::Abstract(
    ///             vec![Type::Integer, Type::Integer],
    ///             Box::new(Type::Integer)
    ///         ),
    ///         location: Location::default(),
    ///     },
    ///     inputs: vec![
    ///         StreamExpression::Constant {
    ///             constant: Constant::Integer(1),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![]),
    ///         },
    ///         StreamExpression::SignalCall {
    ///             id: String::from("x_2"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///             dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
    ///         },
    ///     ],
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
    /// };
    /// assert_eq!(expression, control)
    /// ```
    pub fn normalize(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        self.normalize_root(identifier_creator)
    }

    fn normalize_root(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy { expression, .. } => {
                expression.normalize_cascade(identifier_creator)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication { inputs, .. } => {
                let equations = inputs
                    .iter_mut()
                    .flat_map(|expression| expression.normalize_to_signal_call(identifier_creator))
                    .collect();

                equations
            }
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut equations = expression.normalize_cascade(identifier_creator);
                arms.iter_mut().for_each(|(_, option, body, expression)| {
                    let mut option_equations = option.as_mut().map_or(vec![], |option| {
                        option.normalize_cascade(identifier_creator)
                    });
                    equations.append(&mut option_equations);

                    let mut expression_equations = expression.normalize_cascade(identifier_creator);
                    body.append(&mut expression_equations)
                });
                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut present_equations = present.normalize_cascade(identifier_creator);
                present_body.append(&mut present_equations);

                let mut default_equations = default.normalize_cascade(identifier_creator);
                default_body.append(&mut default_equations);

                option.normalize_cascade(identifier_creator)
            }
            _ => vec![],
        }
    }

    fn normalize_cascade(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy { expression, .. } => {
                expression.normalize_cascade(identifier_creator)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut equations = expression.normalize_cascade(identifier_creator);
                arms.iter_mut().for_each(|(_, option, body, expression)| {
                    let mut option_equations = option.as_mut().map_or(vec![], |option| {
                        option.normalize_cascade(identifier_creator)
                    });
                    equations.append(&mut option_equations);

                    let mut expression_equations = expression.normalize_cascade(identifier_creator);
                    body.append(&mut expression_equations)
                });
                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut present_equations = present.normalize_cascade(identifier_creator);
                present_body.append(&mut present_equations);

                let mut default_equations = default.normalize_cascade(identifier_creator);
                default_body.append(&mut default_equations);

                option.normalize_cascade(identifier_creator)
            }
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication { inputs, .. } => {
                let mut equations = inputs
                    .iter_mut()
                    .flat_map(|expression| expression.normalize_to_signal_call(identifier_creator))
                    .collect::<Vec<_>>();

                let fresh_id = identifier_creator.new_identifier(
                    String::from("x"),
                    String::from(""),
                    String::from(""),
                );

                let typing = self.get_type().clone();
                let location = self.get_location().clone();

                let unitary_node_application_equation = Equation {
                    scope: Scope::Local,
                    signal_type: typing.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                    id: fresh_id.clone(),
                };

                *self = StreamExpression::SignalCall {
                    id: fresh_id.clone(),
                    typing: typing,
                    location: location,
                    dependencies: Dependencies::from(vec![(fresh_id, 0)]),
                };

                equations.push(unitary_node_application_equation);

                equations
            }
            _ => vec![],
        }
    }

    fn normalize_to_signal_call(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
    ) -> Vec<Equation> {
        match self {
            StreamExpression::SignalCall { .. } => vec![],
            _ => {
                let mut equations = self.normalize_cascade(identifier_creator);

                let typing = self.get_type().clone();
                let location = self.get_location().clone();

                let fresh_id = identifier_creator.new_identifier(
                    String::from("x"),
                    String::from(""),
                    String::from(""),
                );

                let new_equation = Equation {
                    scope: Scope::Local,
                    signal_type: typing.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                    id: fresh_id.clone(),
                };

                *self = StreamExpression::SignalCall {
                    id: fresh_id.clone(),
                    typing: typing,
                    location: location,
                    dependencies: Dependencies::from(vec![(fresh_id, 0)]),
                };

                equations.push(new_equation);
                equations
            }
        }
    }

    /// Change node application expressions into unitary node application.
    ///
    /// It removes unused inputs from unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    /// ```
    ///
    /// The application of the node `my_node(g-1, v).o2` is changed
    /// to the application of the unitary node `my_node(v).o2`
    pub fn change_node_application_into_unitary_node_application(
        &mut self,
        unitary_nodes_used_inputs: &HashMap<String, HashMap<String, Vec<bool>>>,
    ) {
        match self {
            StreamExpression::FollowedBy { expression, .. } => expression
                .change_node_application_into_unitary_node_application(unitary_nodes_used_inputs),
            StreamExpression::MapApplication { inputs, .. } => {
                inputs.iter_mut().for_each(|expression| {
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                })
            }
            StreamExpression::NodeApplication {
                node,
                signal,
                inputs,
                typing,
                location,
                dependencies,
            } => {
                let used_inputs = unitary_nodes_used_inputs
                    .get(node)
                    .unwrap()
                    .get(signal)
                    .unwrap();

                let inputs = inputs
                    .into_iter()
                    .zip(used_inputs)
                    .filter(|(_, used)| **used)
                    .map(|(expression, _)| expression.clone())
                    .collect::<Vec<StreamExpression>>();

                *self = StreamExpression::UnitaryNodeApplication {
                    node: node.clone(),
                    signal: signal.clone(),
                    inputs,
                    typing: typing.clone(),
                    location: location.clone(),
                    dependencies: dependencies.clone(),
                };
            }
            StreamExpression::UnitaryNodeApplication { .. } => unreachable!(),
            StreamExpression::Structure { fields, .. } => {
                fields.iter_mut().for_each(|(_, expression)| {
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                })
            }
            StreamExpression::Array { elements, .. } => {
                elements.iter_mut().for_each(|expression| {
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                })
            }
            StreamExpression::Match {
                expression, arms, ..
            } => {
                arms.iter_mut().for_each(|(_, bound, body, expression)| {
                    assert!(body.is_empty());
                    bound.as_mut().map(|expression| {
                        expression.change_node_application_into_unitary_node_application(
                            unitary_nodes_used_inputs,
                        )
                    });
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                });
                expression.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                )
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                assert!(present_body.is_empty() && default_body.is_empty());
                option.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                );
                present.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                );
                default.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                )
            }
            _ => (),
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
                *self = StreamExpression::SignalCall {
                    id: memory_id.clone(),
                    typing: typing.clone(),
                    location: location.clone(),
                    dependencies: Dependencies::from(vec![(memory_id, 0)]),
                }
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.memorize(identifier_creator, memory)),
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication { node, signal, .. } => {
                let memory_id = identifier_creator.new_identifier(
                    String::from("mem"),
                    node.clone(),
                    signal.clone(),
                );
                memory.add_called_node(memory_id, node.clone(), signal.clone())
            }
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
mod change_node_application_into_unitary_node_application {
    use crate::common::{location::Location, r#type::Type};
    use crate::hir::{
        dependencies::Dependencies, expression::Expression, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_change_node_application_to_unitary_node_application() {
        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let unitary_nodes_used_inputs = HashMap::from([(
            String::from("my_node"),
            HashMap::from([
                (String::from("o1"), vec![true, true]),
                (String::from("o2"), vec![false, true]),
            ]),
        )]);

        // expression = my_node(g-1, v).o1
        let mut expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            ],
            signal: String::from("o1"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("g"), 0), (String::from("v"), 0)]),
        };
        expression
            .change_node_application_into_unitary_node_application(&unitary_nodes_used_inputs);

        // control = my_node(g-1, v).o1
        let control = StreamExpression::UnitaryNodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            ],
            signal: String::from("o1"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("g"), 0), (String::from("v"), 0)]),
        };
        assert_eq!(expression, control);
    }

    #[test]
    fn should_remove_unused_inputs_from_to_unitary_node_application() {
        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let unitary_nodes_used_inputs = HashMap::from([(
            String::from("my_node"),
            HashMap::from([
                (String::from("o1"), vec![true, true]),
                (String::from("o2"), vec![false, true]),
            ]),
        )]);

        // expression = my_node(g-1, v).o2
        let mut expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            ],
            signal: String::from("o2"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
        };
        expression
            .change_node_application_into_unitary_node_application(&unitary_nodes_used_inputs);

        // control = my_node(v).o2
        let control = StreamExpression::UnitaryNodeApplication {
            node: String::from("my_node"),
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("v"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
            }],
            signal: String::from("o2"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
        };
        assert_eq!(expression, control);
    }
}

#[cfg(test)]
mod normalize_to_signal_call {
    use std::collections::HashSet;

    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_leave_signal_call_unchanged() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::new(),
        };

        let mut expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
        };
        let equations = expression.normalize_to_signal_call(&mut identifier_creator);

        let control = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
        };
        assert!(equations.is_empty());
        assert_eq!(expression, control)
    }

    #[test]
    fn should_create_signal_call_from_other_expression() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x")]),
        };

        let mut expression = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
        };
        let equations = expression.normalize_to_signal_call(&mut identifier_creator);

        let control = Equation {
            scope: Scope::Local,
            id: String::from("x_1"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        assert_eq!(equations[0], control);

        let control = StreamExpression::SignalCall {
            id: String::from("x_1"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
        };
        assert_eq!(expression, control)
    }
}

#[cfg(test)]
mod normalize {
    use std::collections::HashSet;

    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, expression::Expression,
        identifier_creator::IdentifierCreator, stream_expression::StreamExpression,
    };

    #[test]
    fn should_normalize_expression_according_to_rules() {
        // x: int = 1 + my_node(s, v*2).o;
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };

        let mut expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![]),
                },
                StreamExpression::UnitaryNodeApplication {
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("s"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                        },
                        StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*2"),
                                typing: Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                ),
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
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("s"), 0),
                        (String::from("v"), 0),
                    ]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        let equations = expression.normalize(&mut identifier_creator);

        // x_1: int = v*2;
        // x_2: int = my_node(s, x_1).o;
        let control = vec![
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
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
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
                            dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("s"), 0),
                        (String::from("v"), 0),
                    ]),
                },
                location: Location::default(),
            },
        ];
        assert_eq!(equations, control);

        // x: int = 1 + x_2;
        let control = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![]),
                },
                StreamExpression::SignalCall {
                    id: String::from("x_2"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        assert_eq!(expression, control)
    }
}

#[cfg(test)]
mod memorize {
    use std::collections::HashSet;

    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::{
        dependencies::Dependencies, expression::Expression, identifier_creator::IdentifierCreator,
        memory::Memory, stream_expression::StreamExpression,
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
                typing: Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    id: String::from("s"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                },
                StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        id: String::from("v"),
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
                id: String::from("v"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
            },
        );
        assert_eq!(memory, control);

        let control = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    id: String::from("s"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("mem"),
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
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::SignalCall {
                    id: String::from("s"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("x_1"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                },
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
            String::from("memmy_nodeo"),
            String::from("my_node"),
            String::from("o"),
        );
        assert_eq!(memory, control);

        let control = StreamExpression::UnitaryNodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::SignalCall {
                    id: String::from("s"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("x_1"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                },
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
