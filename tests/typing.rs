use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::{
    equation::Equation, expression::Expression, file::File, node::Node,
    stream_expression::StreamExpression,
};
use grustine::common::{
    constant::Constant,
    location::Location,
    operator::{BinaryOperator, OtherOperator, UnaryOperator},
    r#type::Type,
    scope::Scope,
};
use grustine::parser::langrust;

#[test]
fn file_parser() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let program_test_id = files.add(
        "program_test.gr",
        "
        node counter(res: bool, tick: bool) {
            out o: int = if res then 0 else (0 fby o) + inc;
            inc: int = if tick then 1 else 0;
        } 
        component main() {
            out y: int = counter(false fby (y > 35), half).o;
            half: bool = true fby !half;
        }
        ",
    );

    let mut file: File = langrust::fileParser::new()
        .parse(program_test_id, &files.source(program_test_id).unwrap())
        .unwrap();

    let control_parsed = File {
        typedefs: vec![],
        functions: vec![],
        nodes: vec![Node {
            id: String::from("counter"),
            is_component: false,
            inputs: vec![
                (String::from("res"), Type::Boolean),
                (String::from("tick"), Type::Boolean),
            ],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: OtherOperator::IfThenElse.to_string(),
                                typing: None,
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("res"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(0),
                                    typing: None,
                                    location: Location::default(),
                                },
                                StreamExpression::MapApplication {
                                    function_expression: Expression::Call {
                                        id: BinaryOperator::Add.to_string(),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    inputs: vec![
                                        StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: UnaryOperator::Brackets.to_string(),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            inputs: vec![StreamExpression::FollowedBy {
                                                constant: Constant::Integer(0),
                                                expression: Box::new(
                                                    StreamExpression::SignalCall {
                                                        id: String::from("o"),
                                                        typing: None,
                                                        location: Location::default(),
                                                    },
                                                ),
                                                typing: None,
                                                location: Location::default(),
                                            }],
                                            typing: None,
                                            location: Location::default(),
                                        },
                                        StreamExpression::SignalCall {
                                            id: String::from("inc"),
                                            typing: None,
                                            location: Location::default(),
                                        },
                                    ],
                                    typing: None,
                                    location: Location::default(),
                                },
                            ],
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("inc"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("inc"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: OtherOperator::IfThenElse.to_string(),
                                typing: None,
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("tick"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: None,
                                    location: Location::default(),
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(0),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ],
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        }],
        component: Some(Node {
            id: String::from("main"),
            is_component: true,
            inputs: vec![],
            equations: vec![
                (
                    String::from("y"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("y"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: format!("counter"),
                            inputs: vec![
                                StreamExpression::FollowedBy {
                                    constant: Constant::Boolean(false),
                                    expression: Box::new(StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: UnaryOperator::Brackets.to_string(),
                                            typing: None,
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: BinaryOperator::Grt.to_string(),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::SignalCall {
                                                    id: String::from("y"),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Integer(35),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: None,
                                            location: Location::default(),
                                        }],
                                        typing: None,
                                        location: Location::default(),
                                    }),
                                    typing: None,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("half"),
                                    typing: None,
                                    location: Location::default(),
                                },
                            ],
                            signal: format!("o"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("half"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("half"),
                        signal_type: Type::Boolean,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Boolean(true),
                            expression: Box::new(StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: UnaryOperator::Not.to_string(),
                                    typing: None,
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    id: String::from("half"),
                                    typing: None,
                                    location: Location::default(),
                                }],
                                typing: None,
                                location: Location::default(),
                            }),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        }),
        location: Location::default(),
    };
    assert_eq!(file, control_parsed);

    file.typing(&mut errors).unwrap();

    let control_typed = File {
        typedefs: vec![],
        functions: vec![],
        nodes: vec![Node {
            id: String::from("counter"),
            is_component: false,
            inputs: vec![
                (String::from("res"), Type::Boolean),
                (String::from("tick"), Type::Boolean),
            ],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: OtherOperator::IfThenElse.to_string(),
                                typing: Some(Type::Abstract(
                                    vec![Type::Boolean, Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("res"),
                                    typing: Some(Type::Boolean),
                                    location: Location::default(),
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(0),
                                    typing: Some(Type::Integer),
                                    location: Location::default(),
                                },
                                StreamExpression::MapApplication {
                                    function_expression: Expression::Call {
                                        id: BinaryOperator::Add.to_string(),
                                        typing: Some(Type::Abstract(
                                            vec![Type::Integer, Type::Integer],
                                            Box::new(Type::Integer),
                                        )),
                                        location: Location::default(),
                                    },
                                    inputs: vec![
                                        StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: UnaryOperator::Brackets.to_string(),
                                                typing: Some(Type::Abstract(
                                                    vec![Type::Integer],
                                                    Box::new(Type::Integer),
                                                )),
                                                location: Location::default(),
                                            },
                                            inputs: vec![StreamExpression::FollowedBy {
                                                constant: Constant::Integer(0),
                                                expression: Box::new(
                                                    StreamExpression::SignalCall {
                                                        id: String::from("o"),
                                                        typing: Some(Type::Integer),
                                                        location: Location::default(),
                                                    },
                                                ),
                                                typing: Some(Type::Integer),
                                                location: Location::default(),
                                            }],
                                            typing: Some(Type::Integer),
                                            location: Location::default(),
                                        },
                                        StreamExpression::SignalCall {
                                            id: String::from("inc"),
                                            typing: Some(Type::Integer),
                                            location: Location::default(),
                                        },
                                    ],
                                    typing: Some(Type::Integer),
                                    location: Location::default(),
                                },
                            ],
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("inc"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("inc"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: OtherOperator::IfThenElse.to_string(),
                                typing: Some(Type::Abstract(
                                    vec![Type::Boolean, Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("tick"),
                                    typing: Some(Type::Boolean),
                                    location: Location::default(),
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: Some(Type::Integer),
                                    location: Location::default(),
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(0),
                                    typing: Some(Type::Integer),
                                    location: Location::default(),
                                },
                            ],
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        }],
        component: Some(Node {
            id: String::from("main"),
            is_component: true,
            inputs: vec![],
            equations: vec![
                (
                    String::from("y"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("y"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: format!("counter"),
                            inputs: vec![
                                StreamExpression::FollowedBy {
                                    constant: Constant::Boolean(false),
                                    expression: Box::new(StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: UnaryOperator::Brackets.to_string(),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Boolean],
                                                Box::new(Type::Boolean),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: BinaryOperator::Grt.to_string(),
                                                typing: Some(Type::Abstract(
                                                    vec![Type::Integer, Type::Integer],
                                                    Box::new(Type::Boolean),
                                                )),
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::SignalCall {
                                                    id: String::from("y"),
                                                    typing: Some(Type::Integer),
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Integer(35),
                                                    typing: Some(Type::Integer),
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: Some(Type::Boolean),
                                            location: Location::default(),
                                        }],
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    }),
                                    typing: Some(Type::Boolean),
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("half"),
                                    typing: Some(Type::Boolean),
                                    location: Location::default(),
                                },
                            ],
                            signal: format!("o"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("half"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("half"),
                        signal_type: Type::Boolean,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Boolean(true),
                            expression: Box::new(StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: UnaryOperator::Not.to_string(),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Boolean],
                                        Box::new(Type::Boolean),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    id: String::from("half"),
                                    typing: Some(Type::Boolean),
                                    location: Location::default(),
                                }],
                                typing: Some(Type::Boolean),
                                location: Location::default(),
                            }),
                            typing: Some(Type::Boolean),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        }),
        location: Location::default(),
    };
    assert_eq!(file, control_typed);
}
