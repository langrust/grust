use std::collections::HashSet;

/// Identifier creator used to create fresh signals.
#[derive(Debug, PartialEq)]
pub struct IdentifierCreator {
    /// Already known signals.
    pub signals: HashSet<String>,
}
impl IdentifierCreator {
    /// Create a new identifier creator from a list of identifiers.
    ///
    /// It will store all existing id from the list.
    pub fn from(identifiers: Vec<String>) -> Self {
        let mut signals = HashSet::new();
        identifiers.iter().for_each(|id| {
            signals.insert(id.clone());
        });
        IdentifierCreator { signals }
    }
    fn already_defined(&self, identifier: &String) -> bool {
        self.signals.contains(identifier)
    }
    fn add_signal(&mut self, signal: &str) {
        self.signals.insert(signal.to_string());
    }
    /// Create new identifier from request.
    ///
    /// If the requested identifier is not used then return it.
    /// Otherwise, it create a fresh identifier from this request.
    ///
    /// # Example
    ///
    /// If `mem_x` is requested as new identifier for the node defined bellow,
    /// then it will return it as it is.
    ///
    /// But if it request `mem_x` a second time, then it will return `mem_x_1`.
    ///  
    /// ```GR
    /// node test(i1: int) {
    ///     x: int = i1;
    ///     out o1: int = x;
    /// }
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use grustine::common::{location::Location, scope::Scope, r#type::Type};
    /// use grustine::hir::{
    ///     dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
    ///     memory::Memory, once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
    ///     unitary_node::UnitaryNode,
    /// };
    ///
    /// let unitary_node = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("o1"),
    ///     inputs: vec![(String::from("i1"), Type::Integer)],
    ///     equations: vec![
    ///         Equation {
    ///             scope: Scope::Local,
    ///             id: String::from("x"),
    ///             signal_type: Type::Integer,
    ///             expression: StreamExpression::SignalCall {
    ///                 signal: Signal {
    ///                     id: String::from("i1"),
    ///                     scope: Scope::Input,
    ///                 },
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::new(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///         Equation {
    ///             scope: Scope::Output,
    ///             id: String::from("o1"),
    ///             signal_type: Type::Integer,
    ///             expression: StreamExpression::SignalCall {
    ///                 signal: Signal {
    ///                     id: String::from("x"),
    ///                     scope: Scope::Local,
    ///                 },
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::new(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     memory: Memory::new(),
    ///     location: Location::default(),
    ///     graph: OnceCell::new(),
    /// };
    /// let mut identifier_creator = IdentifierCreator::from(unitary_node.get_signals());
    ///
    /// let identifier = identifier_creator.new_identifier(String::from("mem"), String::from("x"), String::from(""));
    /// let control = String::from("mem_x");
    /// assert_eq!(identifier, control);
    ///
    /// let identifier = identifier_creator.new_identifier(String::from("mem"), String::from("x"), String::from(""));
    /// let control = String::from("mem_x_1");
    /// assert_eq!(identifier, control)
    /// ```
    pub fn new_identifier(
        &mut self,
        mut prefix: String,
        name: String,
        mut suffix: String,
    ) -> String {
        if !(prefix.is_empty() || prefix.ends_with('_')) {
            prefix.push('_');
        }
        if !(suffix.is_empty() || suffix.starts_with('_')) {
            suffix.insert(0, '_');
        }
        let mut identifier = format!("{prefix}{name}{suffix}");

        let mut counter = 1;
        while self.already_defined(&identifier) {
            identifier = format!("{prefix}{name}_{}{suffix}", counter);
            counter += 1;
        }

        self.add_signal(&identifier);
        identifier
    }
}

#[cfg(test)]
mod from {
    use std::collections::HashSet;

    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        memory::Memory, once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };

    #[test]
    fn should_create_identifer_creator_with_all_signals_from_unitary_node() {
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let identifier_creator = IdentifierCreator::from(unitary_node.get_signals());
        let control = IdentifierCreator {
            signals: HashSet::from([String::from("i1"), String::from("o1"), String::from("x")]),
        };

        assert_eq!(identifier_creator, control)
    }
}

#[cfg(test)]
mod new_identifier {
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        memory::Memory, once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };

    #[test]
    fn should_create_the_requested_identifier_when_not_used() {
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut identifier_creator = IdentifierCreator::from(unitary_node.get_signals());
        let identifier = identifier_creator.new_identifier(
            String::from("mem_"),
            String::from("x"),
            String::from(""),
        );

        let control = String::from("mem_x");
        assert_eq!(identifier, control)
    }

    #[test]
    fn should_create_new_identifier_when_used() {
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut identifier_creator = IdentifierCreator::from(unitary_node.get_signals());
        let identifier = identifier_creator.new_identifier(
            String::from(""),
            String::from("x"),
            String::from(""),
        );

        let control = String::from("x_1");
        assert_eq!(identifier, control)
    }

    #[test]
    fn should_create_another_new_identifier_when_used_and_already_created_new_identifier() {
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut identifier_creator = IdentifierCreator::from(unitary_node.get_signals());
        identifier_creator.new_identifier(String::from(""), String::from("x"), String::from(""));
        let identifier = identifier_creator.new_identifier(
            String::from(""),
            String::from("x"),
            String::from(""),
        );

        let control = String::from("x_2");
        assert_eq!(identifier, control)
    }
}
