use crate::common::{location::Location, r#type::Type, scope::Scope};
use crate::hir::{
    identifier_creator::IdentifierCreator, memory::Memory, stream_expression::StreamExpression,
};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust equation AST.
pub struct Equation {
    /// Signal's scope.
    pub scope: Scope,
    /// Identifier of the signal.
    pub id: String,
    /// Signal type.
    pub signal_type: Type,
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    /// Equation location.
    pub location: Location,
}

impl Equation {
    /// Increment memory with equation's expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// An equation `x: int = 0 fby v;` increments memory with the buffer
    /// `mem: int = 0 fby v;` and becomes `x: int = mem;`.
    ///
    /// An equation `x: int = my_node(s, x_1).o;` increments memory with the
    /// node call `memmy_nodeo: (my_node, o);` and the equation is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(&mut self, identifier_creator: &mut IdentifierCreator, memory: &mut Memory) {
        self.expression.memorize(identifier_creator, memory)
    }
}

#[cfg(test)]
mod memorize {
    use std::collections::HashSet;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::{
        equation::Equation, identifier_creator::IdentifierCreator, memory::Memory,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_memorize_followed_by() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };
        let mut memory = Memory::new();

        let mut equation = Equation {
            scope: Scope::Output,
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
                dependencies: Dependencies::from(vec![
                    (String::from("s"), 0),
                    (String::from("v"), 1),
                ]),
            },
            location: Location::default(),
        };
        equation.memorize(&mut identifier_creator, &mut memory);

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

        let control = Equation {
            scope: Scope::Output,
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
                dependencies: Dependencies::from(vec![
                    (String::from("s"), 0),
                    (String::from("v"), 1),
                ]),
            },
            location: Location::default(),
        };
        assert_eq!(equation, control);
    }

    #[test]
    fn should_memorize_node_expression() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };
        let mut memory = Memory::new();

        let mut equation = Equation {
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
                    (String::from("x_1"), 0),
                ]),
            },
            location: Location::default(),
        };
        equation.memorize(&mut identifier_creator, &mut memory);

        let mut control = Memory::new();
        control.add_called_node(
            String::from("memmy_nodeo"),
            String::from("my_node"),
            String::from("o"),
        );
        assert_eq!(memory, control);

        let control = Equation {
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
                    (String::from("x_1"), 0),
                ]),
            },
            location: Location::default(),
        };
        assert_eq!(equation, control);
    }
}
