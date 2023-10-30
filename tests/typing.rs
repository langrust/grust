use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::pattern::Pattern;
use grustine::ast::typedef::Typedef;
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
fn typing_counter() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/counter.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(counter_id, &files.source(counter_id).unwrap())
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

#[test]
fn typing_blinking() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/blinking.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();

    let control_parsed = File {
        typedefs: vec![],
        functions: vec![],
        nodes: vec![
            Node {
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
            },
            Node {
                id: String::from("blinking"),
                is_component: false,
                inputs: vec![(format!("tick_number"), Type::Integer)],
                equations: vec![
                    (
                        String::from("status"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("status"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: OtherOperator::IfThenElse.to_string(),
                                    typing: None,
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: format!("on_off"),
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
                                            StreamExpression::SignalCall {
                                                id: format!("counter"),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            StreamExpression::Constant {
                                                constant: Constant::Integer(1),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                        ],
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
                    (
                        String::from("counter"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("counter"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::NodeApplication {
                                node: format!("counter"),
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("res"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    StreamExpression::Constant {
                                        constant: Constant::Boolean(true),
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
                        format!("res"),
                        Equation {
                            scope: Scope::Local,
                            id: format!("res"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::FollowedBy {
                                constant: Constant::Boolean(true),
                                expression: Box::new(StreamExpression::MapApplication {
                                    function_expression: Expression::Call {
                                        id: UnaryOperator::Brackets.to_string(),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    inputs: vec![StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: BinaryOperator::Eq.to_string(),
                                            typing: None,
                                            location: Location::default(),
                                        },
                                        inputs: vec![
                                            StreamExpression::MapApplication {
                                                function_expression: Expression::Call {
                                                    id: BinaryOperator::Add.to_string(),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                                inputs: vec![
                                                    StreamExpression::SignalCall {
                                                        id: format!("counter"),
                                                        typing: None,
                                                        location: Location::default(),
                                                    },
                                                    StreamExpression::Constant {
                                                        constant: Constant::Integer(1),
                                                        typing: None,
                                                        location: Location::default(),
                                                    },
                                                ],
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            StreamExpression::SignalCall {
                                                id: format!("tick_number"),
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
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("on_off"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("on_off"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Abstraction {
                                    inputs: vec![format!("t"), format!("b")],
                                    expression: Box::new(Expression::Application {
                                        function_expression: Box::new(Expression::Call {
                                            id: OtherOperator::IfThenElse.to_string(),
                                            typing: None,
                                            location: Location::default(),
                                        }),
                                        inputs: vec![
                                            Expression::Call {
                                                id: format!("t"),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            Expression::Application {
                                                function_expression: Box::new(Expression::Call {
                                                    id: UnaryOperator::Not.to_string(),
                                                    typing: None,
                                                    location: Location::default(),
                                                }),
                                                inputs: vec![Expression::Call {
                                                    id: format!("b"),
                                                    typing: None,
                                                    location: Location::default(),
                                                }],
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            Expression::Call {
                                                id: format!("b"),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                        ],
                                        typing: None,
                                        location: Location::default(),
                                    }),
                                    typing: None,
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: format!("res"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    StreamExpression::FollowedBy {
                                        constant: Constant::Boolean(true),
                                        expression: Box::new(StreamExpression::SignalCall {
                                            id: String::from("on_off"),
                                            typing: None,
                                            location: Location::default(),
                                        }),
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
            },
        ],
        component: None,
        location: Location::default(),
    };
    assert_eq!(file.nodes.get(1), control_parsed.nodes.get(1));

    file.typing(&mut errors).unwrap();

    let control_typed = File {
        typedefs: vec![],
        functions: vec![],
        nodes: vec![
            Node {
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
            },
            Node {
                id: String::from("blinking"),
                is_component: false,
                inputs: vec![(format!("tick_number"), Type::Integer)],
                equations: vec![
                    (
                        String::from("status"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("status"),
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
                                        id: format!("on_off"),
                                        typing: Some(Type::Boolean),
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
                                            StreamExpression::SignalCall {
                                                id: format!("counter"),
                                                typing: Some(Type::Integer),
                                                location: Location::default(),
                                            },
                                            StreamExpression::Constant {
                                                constant: Constant::Integer(1),
                                                typing: Some(Type::Integer),
                                                location: Location::default(),
                                            },
                                        ],
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
                    (
                        String::from("counter"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("counter"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::NodeApplication {
                                node: format!("counter"),
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("res"),
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    },
                                    StreamExpression::Constant {
                                        constant: Constant::Boolean(true),
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
                        format!("res"),
                        Equation {
                            scope: Scope::Local,
                            id: format!("res"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::FollowedBy {
                                constant: Constant::Boolean(true),
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
                                            id: BinaryOperator::Eq.to_string(),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer, Type::Integer],
                                                Box::new(Type::Boolean),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![
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
                                                    StreamExpression::SignalCall {
                                                        id: format!("counter"),
                                                        typing: Some(Type::Integer),
                                                        location: Location::default(),
                                                    },
                                                    StreamExpression::Constant {
                                                        constant: Constant::Integer(1),
                                                        typing: Some(Type::Integer),
                                                        location: Location::default(),
                                                    },
                                                ],
                                                typing: Some(Type::Integer),
                                                location: Location::default(),
                                            },
                                            StreamExpression::SignalCall {
                                                id: format!("tick_number"),
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
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("on_off"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("on_off"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::TypedAbstraction {
                                    inputs: vec![
                                        (format!("t"), Type::Boolean),
                                        (format!("b"), Type::Boolean),
                                    ],
                                    expression: Box::new(Expression::Application {
                                        function_expression: Box::new(Expression::Call {
                                            id: OtherOperator::IfThenElse.to_string(),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Boolean, Type::Boolean, Type::Boolean],
                                                Box::new(Type::Boolean),
                                            )),
                                            location: Location::default(),
                                        }),
                                        inputs: vec![
                                            Expression::Call {
                                                id: format!("t"),
                                                typing: Some(Type::Boolean),
                                                location: Location::default(),
                                            },
                                            Expression::Application {
                                                function_expression: Box::new(Expression::Call {
                                                    id: UnaryOperator::Not.to_string(),
                                                    typing: Some(Type::Abstract(
                                                        vec![Type::Boolean],
                                                        Box::new(Type::Boolean),
                                                    )),
                                                    location: Location::default(),
                                                }),
                                                inputs: vec![Expression::Call {
                                                    id: format!("b"),
                                                    typing: Some(Type::Boolean),
                                                    location: Location::default(),
                                                }],
                                                typing: Some(Type::Boolean),
                                                location: Location::default(),
                                            },
                                            Expression::Call {
                                                id: format!("b"),
                                                typing: Some(Type::Boolean),
                                                location: Location::default(),
                                            },
                                        ],
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    }),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Boolean, Type::Boolean],
                                        Box::new(Type::Boolean),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: format!("res"),
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    },
                                    StreamExpression::FollowedBy {
                                        constant: Constant::Boolean(true),
                                        expression: Box::new(StreamExpression::SignalCall {
                                            id: String::from("on_off"),
                                            typing: Some(Type::Boolean),
                                            location: Location::default(),
                                        }),
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    },
                                ],
                                typing: Some(Type::Boolean),
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                ],
                location: Location::default(),
            },
        ],
        component: None,
        location: Location::default(),
    };
    assert_eq!(file, control_typed);
}

#[test]
fn typing_button_management() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/button_management.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();

    let control_parsed = File {
        typedefs: vec![
            Typedef::Enumeration {
                id: format!("Button"),
                elements: vec![format!("Released"), format!("Pressed")],
                location: Location::default(),
            },
            Typedef::Enumeration {
                id: format!("ResetState"),
                elements: vec![format!("InProgress"), format!("Confirmed")],
                location: Location::default(),
            },
        ],
        functions: vec![],
        nodes: vec![
            Node {
                id: String::from("counter"),
                is_component: false,
                inputs: vec![
                    (String::from("res"), Type::Boolean),
                    (String::from("inc"), Type::Integer),
                ],
                equations: vec![(
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
                )],
                location: Location::default(),
            },
            Node {
                id: String::from("reset_button"),
                is_component: false,
                inputs: vec![
                    (
                        format!("button_state"),
                        Type::NotDefinedYet(format!("Button")),
                    ),
                    (format!("period"), Type::Integer),
                    (format!("reset_limit_1"), Type::Integer),
                    (format!("reset_limit_2"), Type::Integer),
                ],
                equations: vec![
                    (
                        String::from("user_reset_request_state"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("user_reset_request_state"),
                            signal_type: Type::NotDefinedYet(format!("ResetState")),
                            expression: StreamExpression::Match {
                                expression: Box::new(StreamExpression::SignalCall {
                                    id: format!("button_state"),
                                    typing: None,
                                    location: Location::default(),
                                }),
                                arms: vec![
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Released"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::Constant {
                                            constant: Constant::Enumeration(
                                                format!("ResetState"),
                                                format!("Confirmed"),
                                            ),
                                            typing: None,
                                            location: Location::default(),
                                        },
                                    ),
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Pressed"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: OtherOperator::IfThenElse.to_string(),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::MapApplication {
                                                    function_expression: Expression::Call {
                                                        id: BinaryOperator::Geq.to_string(),
                                                        typing: None,
                                                        location: Location::default(),
                                                    },
                                                    inputs: vec![
                                                        StreamExpression::SignalCall {
                                                            id: format!("counter"),
                                                            typing: None,
                                                            location: Location::default(),
                                                        },
                                                        StreamExpression::SignalCall {
                                                            id: format!("reset_limit_1"),
                                                            typing: None,
                                                            location: Location::default(),
                                                        },
                                                    ],
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Enumeration(
                                                        format!("ResetState"),
                                                        format!("InProgress"),
                                                    ),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Enumeration(
                                                        format!("ResetState"),
                                                        format!("Confirmed"),
                                                    ),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: None,
                                            location: Location::default(),
                                        },
                                    ),
                                ],
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("user_reset_request"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("user_reset_request"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::Match {
                                expression: Box::new(StreamExpression::SignalCall {
                                    id: format!("button_state"),
                                    typing: None,
                                    location: Location::default(),
                                }),
                                arms: vec![
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Released"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::FollowedBy {
                                            constant: Constant::Boolean(false),
                                            expression: Box::new(StreamExpression::SignalCall {
                                                id: format!("user_reset_request"),
                                                typing: None,
                                                location: Location::default(),
                                            }),
                                            typing: None,
                                            location: Location::default(),
                                        },
                                    ),
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Pressed"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: BinaryOperator::Geq.to_string(),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::SignalCall {
                                                    id: format!("counter"),
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
                                                        StreamExpression::SignalCall {
                                                            id: format!("reset_limit_1"),
                                                            typing: None,
                                                            location: Location::default(),
                                                        },
                                                        StreamExpression::SignalCall {
                                                            id: format!("reset_limit_2"),
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
                                    ),
                                ],
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("counter"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("counter"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::NodeApplication {
                                node: format!("counter"),
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("res"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("period"),
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
                        format!("res"),
                        Equation {
                            scope: Scope::Local,
                            id: format!("res"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: BinaryOperator::And.to_string(),
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
                                        inputs: vec![StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: BinaryOperator::Eq.to_string(),
                                                typing: None,
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::SignalCall {
                                                    id: format!("button_state"),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Enumeration(
                                                        format!("Button"),
                                                        format!("Pressed"),
                                                    ),
                                                    typing: None,
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: None,
                                            location: Location::default(),
                                        }],
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: UnaryOperator::Brackets.to_string(),
                                            typing: None,
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::FollowedBy {
                                            constant: Constant::Boolean(true),
                                            expression: Box::new(
                                                StreamExpression::MapApplication {
                                                    function_expression: Expression::Call {
                                                        id: BinaryOperator::Eq.to_string(),
                                                        typing: None,
                                                        location: Location::default(),
                                                    },
                                                    inputs: vec![
                                                        StreamExpression::SignalCall {
                                                            id: format!("button_state"),
                                                            typing: None,
                                                            location: Location::default(),
                                                        },
                                                        StreamExpression::Constant {
                                                            constant: Constant::Enumeration(
                                                                format!("Button"),
                                                                format!("Released"),
                                                            ),
                                                            typing: None,
                                                            location: Location::default(),
                                                        },
                                                    ],
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
                                ],
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                ],
                location: Location::default(),
            },
        ],
        component: None,
        location: Location::default(),
    };
    assert_eq!(file, control_parsed);

    file.typing(&mut errors).unwrap();

    let control_typed = File {
        typedefs: vec![
            Typedef::Enumeration {
                id: format!("Button"),
                elements: vec![format!("Released"), format!("Pressed")],
                location: Location::default(),
            },
            Typedef::Enumeration {
                id: format!("ResetState"),
                elements: vec![format!("InProgress"), format!("Confirmed")],
                location: Location::default(),
            },
        ],
        functions: vec![],
        nodes: vec![
            Node {
                id: String::from("counter"),
                is_component: false,
                inputs: vec![
                    (String::from("res"), Type::Boolean),
                    (String::from("inc"), Type::Integer),
                ],
                equations: vec![(
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
                )],
                location: Location::default(),
            },
            Node {
                id: String::from("reset_button"),
                is_component: false,
                inputs: vec![
                    (
                        format!("button_state"),
                        Type::Enumeration(format!("Button")),
                    ),
                    (format!("period"), Type::Integer),
                    (format!("reset_limit_1"), Type::Integer),
                    (format!("reset_limit_2"), Type::Integer),
                ],
                equations: vec![
                    (
                        String::from("user_reset_request_state"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("user_reset_request_state"),
                            signal_type: Type::Enumeration(format!("ResetState")),
                            expression: StreamExpression::Match {
                                expression: Box::new(StreamExpression::SignalCall {
                                    id: format!("button_state"),
                                    typing: Some(Type::Enumeration(format!("Button"))),
                                    location: Location::default(),
                                }),
                                arms: vec![
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Released"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::Constant {
                                            constant: Constant::Enumeration(
                                                format!("ResetState"),
                                                format!("Confirmed"),
                                            ),
                                            typing: Some(Type::Enumeration(format!("ResetState"))),
                                            location: Location::default(),
                                        },
                                    ),
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Pressed"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: OtherOperator::IfThenElse.to_string(),
                                                typing: Some(Type::Abstract(
                                                    vec![
                                                        Type::Boolean,
                                                        Type::Enumeration(format!("ResetState")),
                                                        Type::Enumeration(format!("ResetState")),
                                                    ],
                                                    Box::new(Type::Enumeration(format!(
                                                        "ResetState"
                                                    ))),
                                                )),
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::MapApplication {
                                                    function_expression: Expression::Call {
                                                        id: BinaryOperator::Geq.to_string(),
                                                        typing: Some(Type::Abstract(
                                                            vec![Type::Integer, Type::Integer],
                                                            Box::new(Type::Boolean),
                                                        )),
                                                        location: Location::default(),
                                                    },
                                                    inputs: vec![
                                                        StreamExpression::SignalCall {
                                                            id: format!("counter"),
                                                            typing: Some(Type::Integer),
                                                            location: Location::default(),
                                                        },
                                                        StreamExpression::SignalCall {
                                                            id: format!("reset_limit_1"),
                                                            typing: Some(Type::Integer),
                                                            location: Location::default(),
                                                        },
                                                    ],
                                                    typing: Some(Type::Boolean),
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Enumeration(
                                                        format!("ResetState"),
                                                        format!("InProgress"),
                                                    ),
                                                    typing: Some(Type::Enumeration(format!(
                                                        "ResetState"
                                                    ))),
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Enumeration(
                                                        format!("ResetState"),
                                                        format!("Confirmed"),
                                                    ),
                                                    typing: Some(Type::Enumeration(format!(
                                                        "ResetState"
                                                    ))),
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: Some(Type::Enumeration(format!("ResetState"))),
                                            location: Location::default(),
                                        },
                                    ),
                                ],
                                typing: Some(Type::Enumeration(format!("ResetState"))),
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("user_reset_request"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("user_reset_request"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::Match {
                                expression: Box::new(StreamExpression::SignalCall {
                                    id: format!("button_state"),
                                    typing: Some(Type::Enumeration(format!("Button"))),
                                    location: Location::default(),
                                }),
                                arms: vec![
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Released"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::FollowedBy {
                                            constant: Constant::Boolean(false),
                                            expression: Box::new(StreamExpression::SignalCall {
                                                id: format!("user_reset_request"),
                                                typing: Some(Type::Boolean),
                                                location: Location::default(),
                                            }),
                                            typing: Some(Type::Boolean),
                                            location: Location::default(),
                                        },
                                    ),
                                    (
                                        Pattern::Constant {
                                            constant: Constant::Enumeration(
                                                format!("Button"),
                                                format!("Pressed"),
                                            ),
                                            location: Location::default(),
                                        },
                                        None,
                                        StreamExpression::MapApplication {
                                            function_expression: Expression::Call {
                                                id: BinaryOperator::Geq.to_string(),
                                                typing: Some(Type::Abstract(
                                                    vec![Type::Integer, Type::Integer],
                                                    Box::new(Type::Boolean),
                                                )),
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::SignalCall {
                                                    id: format!("counter"),
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
                                                        StreamExpression::SignalCall {
                                                            id: format!("reset_limit_1"),
                                                            typing: Some(Type::Integer),
                                                            location: Location::default(),
                                                        },
                                                        StreamExpression::SignalCall {
                                                            id: format!("reset_limit_2"),
                                                            typing: Some(Type::Integer),
                                                            location: Location::default(),
                                                        },
                                                    ],
                                                    typing: Some(Type::Integer),
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: Some(Type::Boolean),
                                            location: Location::default(),
                                        },
                                    ),
                                ],
                                typing: Some(Type::Boolean),
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("counter"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("counter"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::NodeApplication {
                                node: format!("counter"),
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("res"),
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("period"),
                                        typing: Some(Type::Integer),
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
                        format!("res"),
                        Equation {
                            scope: Scope::Local,
                            id: format!("res"),
                            signal_type: Type::Boolean,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: BinaryOperator::And.to_string(),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Boolean, Type::Boolean],
                                        Box::new(Type::Boolean),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::MapApplication {
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
                                                id: BinaryOperator::Eq.to_string(),
                                                typing: Some(Type::Abstract(
                                                    vec![
                                                        Type::Enumeration(format!("Button")),
                                                        Type::Enumeration(format!("Button")),
                                                    ],
                                                    Box::new(Type::Boolean),
                                                )),
                                                location: Location::default(),
                                            },
                                            inputs: vec![
                                                StreamExpression::SignalCall {
                                                    id: format!("button_state"),
                                                    typing: Some(Type::Enumeration(format!(
                                                        "Button"
                                                    ))),
                                                    location: Location::default(),
                                                },
                                                StreamExpression::Constant {
                                                    constant: Constant::Enumeration(
                                                        format!("Button"),
                                                        format!("Pressed"),
                                                    ),
                                                    typing: Some(Type::Enumeration(format!(
                                                        "Button"
                                                    ))),
                                                    location: Location::default(),
                                                },
                                            ],
                                            typing: Some(Type::Boolean),
                                            location: Location::default(),
                                        }],
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    },
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: UnaryOperator::Brackets.to_string(),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Boolean],
                                                Box::new(Type::Boolean),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::FollowedBy {
                                            constant: Constant::Boolean(true),
                                            expression: Box::new(
                                                StreamExpression::MapApplication {
                                                    function_expression: Expression::Call {
                                                        id: BinaryOperator::Eq.to_string(),
                                                        typing: Some(Type::Abstract(
                                                            vec![
                                                                Type::Enumeration(format!(
                                                                    "Button"
                                                                )),
                                                                Type::Enumeration(format!(
                                                                    "Button"
                                                                )),
                                                            ],
                                                            Box::new(Type::Boolean),
                                                        )),
                                                        location: Location::default(),
                                                    },
                                                    inputs: vec![
                                                        StreamExpression::SignalCall {
                                                            id: format!("button_state"),
                                                            typing: Some(Type::Enumeration(
                                                                format!("Button"),
                                                            )),
                                                            location: Location::default(),
                                                        },
                                                        StreamExpression::Constant {
                                                            constant: Constant::Enumeration(
                                                                format!("Button"),
                                                                format!("Released"),
                                                            ),
                                                            typing: Some(Type::Enumeration(
                                                                format!("Button"),
                                                            )),
                                                            location: Location::default(),
                                                        },
                                                    ],
                                                    typing: Some(Type::Boolean),
                                                    location: Location::default(),
                                                },
                                            ),
                                            typing: Some(Type::Boolean),
                                            location: Location::default(),
                                        }],
                                        typing: Some(Type::Boolean),
                                        location: Location::default(),
                                    },
                                ],
                                typing: Some(Type::Boolean),
                                location: Location::default(),
                            },
                            location: Location::default(),
                        },
                    ),
                ],
                location: Location::default(),
            },
        ],
        component: None,
        location: Location::default(),
    };
    assert_eq!(file, control_typed);
}
