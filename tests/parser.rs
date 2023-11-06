use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::file::File;
use grustine::parser::langrust;

#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::{Files, SimpleFiles};

    use grustine::ast::{
        equation::Equation, expression::Expression, file::File, function::Function, node::Node,
        statement::Statement, stream_expression::StreamExpression, typedef::Typedef,
    };
    use grustine::common::{
        constant::Constant,
        location::Location,
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        pattern::Pattern,
        r#type::Type,
        scope::Scope,
    };
    use grustine::parser::langrust;

    #[test]
    fn file_parser() {
        let mut files = SimpleFiles::new();

        let module_test_id = files.add(
            "module_test.gr",
            "function test(i: int) -> int {let x: int = i; let o: int = x; return o;} 
                node test(i: int){out o: int = x; x: int = i;}
                enum Color { Red, Blue, Green, Yellow }
                node test(i: int){out o: int = x; x: int = i;}
                function test(i: int) -> int {let x: int = i; let o: int = x; return o;}
                node test(i: int){out o: int = x; x: int = i;}",
        );
        let program_test_id = files.add(
            "program_test.gr",
            "node test(i: int){out o: int = x; x: int = i;} 
                component test(i: int){out o: int = x; x: int = i;}
                array Matrix [[int; 3]; 3]
                node test(i: int){out o: int = x; x: int = i;}
                function test(i: int) -> int {let x: int = i; let o: int = x; return o;}
                struct Point {x: int, y: int, }
                function test(i: int) -> int {let x: int = i; let o: int = x; return o;}",
        );

        let file = langrust::fileParser::new()
            .parse(module_test_id, &files.source(module_test_id).unwrap())
            .unwrap();
        assert_eq!(
            file,
            File {
                typedefs: vec![Typedef::Enumeration {
                    id: String::from("Color"),
                    elements: vec![
                        String::from("Red"),
                        String::from("Blue"),
                        String::from("Green"),
                        String::from("Yellow"),
                    ],
                    location: Location::default(),
                }],
                functions: vec![
                    Function {
                        id: String::from("test"),
                        inputs: vec![(String::from("i"), Type::Integer)],
                        statements: vec![
                            Statement {
                                id: String::from("x"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("i"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                            Statement {
                                id: String::from("o"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                        ],
                        returned: (
                            Type::Integer,
                            Expression::Call {
                                id: String::from("o"),
                                typing: None,
                                location: Location::default(),
                            },
                        ),
                        location: Location::default(),
                    },
                    Function {
                        id: String::from("test"),
                        inputs: vec![(String::from("i"), Type::Integer)],
                        statements: vec![
                            Statement {
                                id: String::from("x"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("i"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                            Statement {
                                id: String::from("o"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                        ],
                        returned: (
                            Type::Integer,
                            Expression::Call {
                                id: String::from("o"),
                                typing: None,
                                location: Location::default(),
                            },
                        ),
                        location: Location::default(),
                    }
                ],
                nodes: vec![
                    Node {
                        id: String::from("test"),
                        is_component: false,
                        inputs: vec![(String::from("i"), Type::Integer)],
                        equations: vec![
                            (
                                String::from("o"),
                                Equation {
                                    scope: Scope::Output,
                                    id: String::from("o"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            ),
                            (
                                String::from("x"),
                                Equation {
                                    scope: Scope::Local,
                                    id: String::from("x"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("i"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            )
                        ],
                        location: Location::default(),
                    },
                    Node {
                        id: String::from("test"),
                        is_component: false,
                        inputs: vec![(String::from("i"), Type::Integer)],
                        equations: vec![
                            (
                                String::from("o"),
                                Equation {
                                    scope: Scope::Output,
                                    id: String::from("o"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            ),
                            (
                                String::from("x"),
                                Equation {
                                    scope: Scope::Local,
                                    id: String::from("x"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("i"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            )
                        ],
                        location: Location::default(),
                    },
                    Node {
                        id: String::from("test"),
                        is_component: false,
                        inputs: vec![(String::from("i"), Type::Integer)],
                        equations: vec![
                            (
                                String::from("o"),
                                Equation {
                                    scope: Scope::Output,
                                    id: String::from("o"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            ),
                            (
                                String::from("x"),
                                Equation {
                                    scope: Scope::Local,
                                    id: String::from("x"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("i"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            )
                        ],
                        location: Location::default(),
                    }
                ],
                component: None,
                location: Location::default()
            },
        );

        let file = langrust::fileParser::new()
            .parse(program_test_id, &files.source(program_test_id).unwrap())
            .unwrap();
        assert_eq!(
            file,
            File {
                typedefs: vec![
                    Typedef::Array {
                        id: String::from("Matrix"),
                        array_type: Type::Array(Box::new(Type::Integer), 3),
                        size: 3,
                        location: Location::default(),
                    },
                    Typedef::Structure {
                        id: String::from("Point"),
                        fields: vec![
                            (String::from("x"), Type::Integer),
                            (String::from("y"), Type::Integer),
                        ],
                        location: Location::default(),
                    }
                ],
                functions: vec![
                    Function {
                        id: String::from("test"),
                        inputs: vec![(String::from("i"), Type::Integer)],
                        statements: vec![
                            Statement {
                                id: String::from("x"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("i"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                            Statement {
                                id: String::from("o"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                        ],
                        returned: (
                            Type::Integer,
                            Expression::Call {
                                id: String::from("o"),
                                typing: None,
                                location: Location::default(),
                            },
                        ),
                        location: Location::default(),
                    },
                    Function {
                        id: String::from("test"),
                        inputs: vec![(String::from("i"), Type::Integer)],
                        statements: vec![
                            Statement {
                                id: String::from("x"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("i"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                            Statement {
                                id: String::from("o"),
                                element_type: Type::Integer,
                                expression: Expression::Call {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            },
                        ],
                        returned: (
                            Type::Integer,
                            Expression::Call {
                                id: String::from("o"),
                                typing: None,
                                location: Location::default(),
                            },
                        ),
                        location: Location::default(),
                    }
                ],
                nodes: vec![
                    Node {
                        id: String::from("test"),
                        is_component: false,
                        inputs: vec![(String::from("i"), Type::Integer)],
                        equations: vec![
                            (
                                String::from("o"),
                                Equation {
                                    scope: Scope::Output,
                                    id: String::from("o"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            ),
                            (
                                String::from("x"),
                                Equation {
                                    scope: Scope::Local,
                                    id: String::from("x"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("i"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            )
                        ],
                        location: Location::default(),
                    },
                    Node {
                        id: String::from("test"),
                        is_component: false,
                        inputs: vec![(String::from("i"), Type::Integer)],
                        equations: vec![
                            (
                                String::from("o"),
                                Equation {
                                    scope: Scope::Output,
                                    id: String::from("o"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            ),
                            (
                                String::from("x"),
                                Equation {
                                    scope: Scope::Local,
                                    id: String::from("x"),
                                    signal_type: Type::Integer,
                                    expression: StreamExpression::SignalCall {
                                        id: String::from("i"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    location: Location::default(),
                                }
                            )
                        ],
                        location: Location::default(),
                    }
                ],
                component: Some(Node {
                    id: String::from("test"),
                    is_component: true,
                    inputs: vec![(String::from("i"), Type::Integer)],
                    equations: vec![
                        (
                            String::from("o"),
                            Equation {
                                scope: Scope::Output,
                                id: String::from("o"),
                                signal_type: Type::Integer,
                                expression: StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            }
                        ),
                        (
                            String::from("x"),
                            Equation {
                                scope: Scope::Local,
                                id: String::from("x"),
                                signal_type: Type::Integer,
                                expression: StreamExpression::SignalCall {
                                    id: String::from("i"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                location: Location::default(),
                            }
                        )
                    ],
                    location: Location::default(),
                }),
                location: Location::default()
            },
        );
    }

    #[test]
    fn user_types_parser() {
        let mut files = SimpleFiles::new();

        let struct_test_id = files.add("struct_test.gr", "struct Point {x: int, y: int, }");
        let enum_test_id = files.add("enum_test.gr", "enum Color { Red, Blue, Green, Yellow }");
        let array_test_id = files.add("array_test.gr", "array Matrix [[int; 3]; 3]");

        let user_type = langrust::userTypeParser::new()
            .parse(struct_test_id, &files.source(struct_test_id).unwrap())
            .unwrap();
        assert_eq!(
            user_type,
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            }
        );
        let user_type = langrust::userTypeParser::new()
            .parse(enum_test_id, &files.source(enum_test_id).unwrap())
            .unwrap();
        assert_eq!(
            user_type,
            Typedef::Enumeration {
                id: String::from("Color"),
                elements: vec![
                    String::from("Red"),
                    String::from("Blue"),
                    String::from("Green"),
                    String::from("Yellow"),
                ],
                location: Location::default(),
            }
        );
        let user_type = langrust::userTypeParser::new()
            .parse(array_test_id, &files.source(array_test_id).unwrap())
            .unwrap();
        assert_eq!(
            user_type,
            Typedef::Array {
                id: String::from("Matrix"),
                array_type: Type::Array(Box::new(Type::Integer), 3),
                size: 3,
                location: Location::default(),
            }
        );
    }

    #[test]
    fn component_parser() {
        let mut files = SimpleFiles::new();

        let component_test_id = files.add(
            "component_test.gr",
            "component test(i: int){out o: int = x; x: int = i;}",
        );

        let component = langrust::componentParser::new()
            .parse(component_test_id, &files.source(component_test_id).unwrap())
            .unwrap();
        assert_eq!(
            component,
            Node {
                id: String::from("test"),
                is_component: true,
                inputs: vec![(String::from("i"), Type::Integer)],
                equations: vec![
                    (
                        String::from("o"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("o"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }
                    ),
                    (
                        String::from("x"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("x"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::SignalCall {
                                id: String::from("i"),
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }
                    )
                ],
                location: Location::default(),
            },
        );
    }

    #[test]
    fn node_parser() {
        let mut files = SimpleFiles::new();

        let node_test_id = files.add(
            "node_test.gr",
            "node test(i: int){out o: int = x; x: int = i;}",
        );

        let node = langrust::nodeParser::new()
            .parse(node_test_id, &files.source(node_test_id).unwrap())
            .unwrap();
        assert_eq!(
            node,
            Node {
                id: String::from("test"),
                is_component: false,
                inputs: vec![(String::from("i"), Type::Integer)],
                equations: vec![
                    (
                        String::from("o"),
                        Equation {
                            scope: Scope::Output,
                            id: String::from("o"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }
                    ),
                    (
                        String::from("x"),
                        Equation {
                            scope: Scope::Local,
                            id: String::from("x"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::SignalCall {
                                id: String::from("i"),
                                typing: None,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }
                    )
                ],
                location: Location::default(),
            },
        );
    }

    #[test]
    fn function_parser() {
        let mut files = SimpleFiles::new();

        let function_test_id = files.add(
            "function_test.gr",
            "function test(i: int) -> int {let x: int = i; let o: int = x; return o;}",
        );

        let function = langrust::functionParser::new()
            .parse(function_test_id, &files.source(function_test_id).unwrap())
            .unwrap();
        assert_eq!(
            function,
            Function {
                id: String::from("test"),
                inputs: vec![(String::from("i"), Type::Integer)],
                statements: vec![
                    Statement {
                        id: String::from("x"),
                        element_type: Type::Integer,
                        expression: Expression::Call {
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                    Statement {
                        id: String::from("o"),
                        element_type: Type::Integer,
                        expression: Expression::Call {
                            id: String::from("x"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ],
                returned: (
                    Type::Integer,
                    Expression::Call {
                        id: String::from("o"),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                location: Location::default(),
            },
        );
    }

    #[test]
    fn equation() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("equation_test.gr", "c: Color = Color.Yellow;");
        let file_id2 = files
            .add(
                "equation_match_test.gr",
                "out compare: int = match (a) { Point {x: 0, y: _} => 0, Point {x: x, y: _} if x < 0 => -1, _ => 1 };"
            );

        let equation = langrust::equationParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Equation {
                scope: Scope::Local,
                id: String::from("c"),
                signal_type: Type::NotDefinedYet(String::from("Color")),
                expression: StreamExpression::Constant {
                    constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
                    typing: None,
                    location: Location::default()
                },
                location: Location::default(),
            },
            equation
        );
        let equation = langrust::equationParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            Equation {
                scope: Scope::Output,
                id: String::from("compare"),
                signal_type: Type::Integer,
                expression: StreamExpression::Match {
                    expression: Box::new(StreamExpression::SignalCall {
                        id: String::from("a"),
                        typing: None,
                        location: Location::default()
                    }),
                    arms: vec![
                        (
                            Pattern::Structure {
                                name: String::from("Point"),
                                fields: vec![
                                    (
                                        String::from("x"),
                                        Pattern::Constant {
                                            constant: Constant::Integer(0),
                                            location: Location::default()
                                        }
                                    ),
                                    (
                                        String::from("y"),
                                        Pattern::Default {
                                            location: Location::default()
                                        }
                                    )
                                ],
                                location: Location::default()
                            },
                            None,
                            StreamExpression::Constant {
                                constant: Constant::Integer(0),
                                typing: None,
                                location: Location::default()
                            }
                        ),
                        (
                            Pattern::Structure {
                                name: String::from("Point"),
                                fields: vec![
                                    (
                                        String::from("x"),
                                        Pattern::Identifier {
                                            name: String::from("x"),
                                            location: Location::default()
                                        }
                                    ),
                                    (
                                        String::from("y"),
                                        Pattern::Default {
                                            location: Location::default()
                                        }
                                    )
                                ],
                                location: Location::default()
                            },
                            Some(StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: BinaryOperator::Low.to_string(),
                                    typing: None,
                                    location: Location::default()
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default()
                                    },
                                    StreamExpression::Constant {
                                        constant: Constant::Integer(0),
                                        typing: None,
                                        location: Location::default()
                                    }
                                ],
                                typing: None,
                                location: Location::default()
                            }),
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: UnaryOperator::Neg.to_string(),
                                    typing: None,
                                    location: Location::default()
                                },
                                inputs: vec![StreamExpression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: None,
                                    location: Location::default()
                                }],
                                typing: None,
                                location: Location::default()
                            }
                        ),
                        (
                            Pattern::Default {
                                location: Location::default()
                            },
                            None,
                            StreamExpression::Constant {
                                constant: Constant::Integer(1),
                                typing: None,
                                location: Location::default()
                            }
                        )
                    ],
                    typing: None,
                    location: Location::default()
                },
                location: Location::default(),
            },
            equation
        );
    }

    #[test]
    fn statement() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("statement_test.gr", "let c: Color = Color.Yellow;");
        let file_id2 = files
            .add(
                "statement_match_test.gr",
                "let compare: int = match (a) { Point {x: 0, y: _} => 0, Point {x: x, y: _} if x < 0 => -1, _ => 1 };"
            );

        let statement = langrust::statementParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Statement {
                id: String::from("c"),
                element_type: Type::NotDefinedYet(String::from("Color")),
                expression: Expression::Constant {
                    constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
                    typing: None,
                    location: Location::default()
                },
                location: Location::default(),
            },
            statement
        );
        let statement = langrust::statementParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            Statement {
                id: String::from("compare"),
                element_type: Type::Integer,
                expression: Expression::Match {
                    expression: Box::new(Expression::Call {
                        id: String::from("a"),
                        typing: None,
                        location: Location::default()
                    }),
                    arms: vec![
                        (
                            Pattern::Structure {
                                name: String::from("Point"),
                                fields: vec![
                                    (
                                        String::from("x"),
                                        Pattern::Constant {
                                            constant: Constant::Integer(0),
                                            location: Location::default()
                                        }
                                    ),
                                    (
                                        String::from("y"),
                                        Pattern::Default {
                                            location: Location::default()
                                        }
                                    )
                                ],
                                location: Location::default()
                            },
                            None,
                            Expression::Constant {
                                constant: Constant::Integer(0),
                                typing: None,
                                location: Location::default()
                            }
                        ),
                        (
                            Pattern::Structure {
                                name: String::from("Point"),
                                fields: vec![
                                    (
                                        String::from("x"),
                                        Pattern::Identifier {
                                            name: String::from("x"),
                                            location: Location::default()
                                        }
                                    ),
                                    (
                                        String::from("y"),
                                        Pattern::Default {
                                            location: Location::default()
                                        }
                                    )
                                ],
                                location: Location::default()
                            },
                            Some(Expression::Application {
                                function_expression: Box::new(Expression::Call {
                                    id: BinaryOperator::Low.to_string(),
                                    typing: None,
                                    location: Location::default()
                                }),
                                inputs: vec![
                                    Expression::Call {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default()
                                    },
                                    Expression::Constant {
                                        constant: Constant::Integer(0),
                                        typing: None,
                                        location: Location::default()
                                    }
                                ],
                                typing: None,
                                location: Location::default()
                            }),
                            Expression::Application {
                                function_expression: Box::new(Expression::Call {
                                    id: UnaryOperator::Neg.to_string(),
                                    typing: None,
                                    location: Location::default()
                                }),
                                inputs: vec![Expression::Constant {
                                    constant: Constant::Integer(1),
                                    typing: None,
                                    location: Location::default()
                                }],
                                typing: None,
                                location: Location::default()
                            }
                        ),
                        (
                            Pattern::Default {
                                location: Location::default()
                            },
                            None,
                            Expression::Constant {
                                constant: Constant::Integer(1),
                                typing: None,
                                location: Location::default()
                            }
                        )
                    ],
                    typing: None,
                    location: Location::default()
                },
                location: Location::default(),
            },
            statement
        );
    }

    #[test]
    fn types() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("int_test.gr", "int");
        let file_id2 = files.add("float_test.gr", "float");
        let file_id3 = files.add("bool_test.gr", "bool");
        let file_id4 = files.add("string_test.gr", "string");
        let file_id5 = files.add("unit_test.gr", "unit");
        let file_id6 = files.add("array_test.gr", "[int; 3]");
        let file_id7 = files.add("option_test.gr", "int?");
        let file_id8 = files.add("undefined_type_test.gr", "Color");

        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::Integer);
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::Float);
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::Boolean);
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::String);
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::Unit);
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::Array(Box::new(Type::Integer), 3));
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::Option(Box::new(Type::Integer)));
        let basic_type = langrust::basicTypeParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(basic_type, Type::NotDefinedYet(String::from("Color")));
    }

    #[test]
    fn pattern() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("identifier_test.gr", "x");
        let file_id2 = files.add("constant_test.gr", "Color.Yellow");
        let file_id3 = files.add("structure_test.gr", "Point { x: 0, y: _}");
        let file_id4 = files.add("some_test.gr", "some(value)");
        let file_id5 = files.add("none_test.gr", "none");
        let file_id6 = files.add("default_test.gr", "_");

        let pattern = langrust::patternParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Pattern::Identifier {
                name: String::from("x"),
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            Pattern::Constant {
                constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            Pattern::Structure {
                name: String::from("Point"),
                fields: vec![
                    (
                        String::from("x"),
                        Pattern::Constant {
                            constant: Constant::Integer(0),
                            location: Location::default()
                        }
                    ),
                    (
                        String::from("y"),
                        Pattern::Default {
                            location: Location::default()
                        }
                    )
                ],
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            Pattern::Some {
                pattern: Box::new(Pattern::Identifier {
                    name: String::from("value"),
                    location: Location::default()
                }),
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            Pattern::None {
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            Pattern::Default {
                location: Location::default()
            },
            pattern
        );
    }

    #[test]
    fn stream_expression() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("constant_test.gr", "Color.Yellow");
        let file_id2 = files.add("signal_call_test.gr", "x");
        let file_id3 = files.add("brackets_test.gr", "(3)");
        let file_id4 = files.add("unary_test.gr", "-3");
        let file_id5 = files.add("binary_test.gr", "4*5-3");
        let file_id6 = files.add("map_application_test.gr", "(x*y).map(sqrt)");
        let file_id7 = files.add("print_test.gr", "print(\"Hello world\")");
        let file_id8 = files.add(
            "node_application_test.gr",
            "my_node(my_input1, my_input2).my_signal",
        );
        let file_id9 = files.add("fby_test.gr", "0 fby x + 1");
        let file_id10 = files.add("ifthenelse_test.gr", "if b then x else y");
        let file_id11 = files.add("struct_test.gr", "Point { x: 3, y: 0, }");
        let file_id12 = files.add("array_test.gr", "[1, 2, 3]");
        let file_id13 = files.add("unified_array_test.gr", "[0.01; 3]");
        let file_id14 = files.add("when_id_test.gr", "when a = x then a else 0");
        let file_id15 = files.add("when_test.gr", "when a then a else 0");
        let file_id16 = files.add(
            "match_test.gr",
            "match (a) { Point {x: 0, y: _} => 0, Point {x: x, y: _} if x < 0 => -1, _ => 1 }",
        );

        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::Constant {
                constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: UnaryOperator::Brackets.to_string(),
                    typing: None,
                    location: Location::default()
                },
                inputs: vec![StreamExpression::Constant {
                    constant: Constant::Integer(3),
                    typing: None,
                    location: Location::default()
                },],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: UnaryOperator::Neg.to_string(),
                    typing: None,
                    location: Location::default()
                },
                inputs: vec![StreamExpression::Constant {
                    constant: Constant::Integer(3),
                    typing: None,
                    location: Location::default()
                },],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: BinaryOperator::Sub.to_string(),
                    typing: None,
                    location: Location::default()
                },
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: BinaryOperator::Mul.to_string(),
                            typing: None,
                            location: Location::default()
                        },
                        inputs: vec![
                            StreamExpression::Constant {
                                constant: Constant::Integer(4),
                                typing: None,
                                location: Location::default()
                            },
                            StreamExpression::Constant {
                                constant: Constant::Integer(5),
                                typing: None,
                                location: Location::default()
                            },
                        ],
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(3),
                        typing: None,
                        location: Location::default()
                    },
                ],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("sqrt"),
                    typing: None,
                    location: Location::default()
                },
                inputs: vec![StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: BinaryOperator::Mul.to_string(),
                        typing: None,
                        location: Location::default()
                    },
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: None,
                            location: Location::default()
                        },
                        StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: None,
                            location: Location::default()
                        },
                    ],
                    typing: None,
                    location: Location::default()
                },],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: OtherOperator::Print.to_string(),
                    typing: None,
                    location: Location::default()
                },
                inputs: vec![StreamExpression::Constant {
                    constant: Constant::String(String::from("Hello world")),
                    typing: None,
                    location: Location::default()
                }],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::NodeApplication {
                node: String::from("my_node"),
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: String::from("my_input1"),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::SignalCall {
                        id: String::from("my_input2"),
                        typing: None,
                        location: Location::default()
                    }
                ],
                signal: String::from("my_signal"),
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: BinaryOperator::Add.to_string(),
                        typing: None,
                        location: Location::default()
                    },
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: None,
                            location: Location::default()
                        },
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: None,
                            location: Location::default()
                        },
                    ],
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: OtherOperator::IfThenElse.to_string(),
                    typing: None,
                    location: Location::default()
                },
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: String::from("b"),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::SignalCall {
                        id: String::from("y"),
                        typing: None,
                        location: Location::default()
                    },
                ],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::Structure {
                name: String::from("Point"),
                fields: vec![
                    (
                        String::from("x"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(3),
                            typing: None,
                            location: Location::default()
                        }
                    ),
                    (
                        String::from("y"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(0),
                            typing: None,
                            location: Location::default()
                        }
                    )
                ],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id12, &files.source(file_id12).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::Array {
                elements: vec![
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(3),
                        typing: None,
                        location: Location::default()
                    }
                ],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id13, &files.source(file_id13).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::Array {
                elements: vec![
                    StreamExpression::Constant {
                        constant: Constant::Float(0.01),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Float(0.01),
                        typing: None,
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Float(0.01),
                        typing: None,
                        location: Location::default()
                    }
                ],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id14, &files.source(file_id14).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::When {
                id: String::from("a"),
                option: Box::new(StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default()
                }),
                present: Box::new(StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                default: Box::new(StreamExpression::Constant {
                    constant: Constant::Integer(0),
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id15, &files.source(file_id15).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::When {
                id: String::from("a"),
                option: Box::new(StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                present: Box::new(StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                default: Box::new(StreamExpression::Constant {
                    constant: Constant::Integer(0),
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id16, &files.source(file_id16).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::Match {
                expression: Box::new(StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                arms: vec![
                    (
                        Pattern::Structure {
                            name: String::from("Point"),
                            fields: vec![
                                (
                                    String::from("x"),
                                    Pattern::Constant {
                                        constant: Constant::Integer(0),
                                        location: Location::default()
                                    }
                                ),
                                (
                                    String::from("y"),
                                    Pattern::Default {
                                        location: Location::default()
                                    }
                                )
                            ],
                            location: Location::default()
                        },
                        None,
                        StreamExpression::Constant {
                            constant: Constant::Integer(0),
                            typing: None,
                            location: Location::default()
                        }
                    ),
                    (
                        Pattern::Structure {
                            name: String::from("Point"),
                            fields: vec![
                                (
                                    String::from("x"),
                                    Pattern::Identifier {
                                        name: String::from("x"),
                                        location: Location::default()
                                    }
                                ),
                                (
                                    String::from("y"),
                                    Pattern::Default {
                                        location: Location::default()
                                    }
                                )
                            ],
                            location: Location::default()
                        },
                        Some(StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: BinaryOperator::Low.to_string(),
                                typing: None,
                                location: Location::default()
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default()
                                },
                                StreamExpression::Constant {
                                    constant: Constant::Integer(0),
                                    typing: None,
                                    location: Location::default()
                                }
                            ],
                            typing: None,
                            location: Location::default()
                        }),
                        StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: UnaryOperator::Neg.to_string(),
                                typing: None,
                                location: Location::default()
                            },
                            inputs: vec![StreamExpression::Constant {
                                constant: Constant::Integer(1),
                                typing: None,
                                location: Location::default()
                            }],
                            typing: None,
                            location: Location::default()
                        }
                    ),
                    (
                        Pattern::Default {
                            location: Location::default()
                        },
                        None,
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: None,
                            location: Location::default()
                        }
                    )
                ],
                typing: None,
                location: Location::default()
            },
            stream_expression
        );
    }

    #[test]
    fn expression() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("constant_test.gr", "Color.Yellow");
        let file_id2 = files.add("element_call_test.gr", "x");
        let file_id3 = files.add("brackets_test.gr", "(3)");
        let file_id4 = files.add("unary_test.gr", "-3");
        let file_id5 = files.add("binary_test.gr", "4*5-3");
        let file_id6 = files.add("function_application_test.gr", "sqrt(4*5-3)");
        let file_id7 = files.add("print_test.gr", "print(\"Hello world\")");
        let file_id8 = files.add("abstraction_test.gr", "|x, y| x + y");
        let file_id9 = files.add("typed_abstraction_test.gr", "|x: int, y: int| x + y");
        let file_id10 = files.add("ifthenelse_test.gr", "if b then x else y");
        let file_id11 = files.add("struct_test.gr", "Point { x: 3, y: 0, }");
        let file_id12 = files.add("array_test.gr", "[1, 2, 3]");
        let file_id13 = files.add("unified_array_test.gr", "[0.01; 3]");
        let file_id14 = files.add("when_id_test.gr", "when a = x then a else 0");
        let file_id15 = files.add("when_test.gr", "when a then a else 0");
        let file_id16 = files.add(
            "match_test.gr",
            "match (a) { Point {x: 0, y: _} => 0, Point {x: x, y: _} if x < 0 => -1, _ => 1 }",
        );

        let expression = langrust::expressionParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Constant {
                constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: UnaryOperator::Brackets.to_string(),
                    typing: None,
                    location: Location::default()
                }),
                inputs: vec![Expression::Constant {
                    constant: Constant::Integer(3),
                    typing: None,
                    location: Location::default()
                },],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: UnaryOperator::Neg.to_string(),
                    typing: None,
                    location: Location::default()
                }),
                inputs: vec![Expression::Constant {
                    constant: Constant::Integer(3),
                    typing: None,
                    location: Location::default()
                },],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: BinaryOperator::Sub.to_string(),
                    typing: None,
                    location: Location::default()
                }),
                inputs: vec![
                    Expression::Application {
                        function_expression: Box::new(Expression::Call {
                            id: BinaryOperator::Mul.to_string(),
                            typing: None,
                            location: Location::default()
                        }),
                        inputs: vec![
                            Expression::Constant {
                                constant: Constant::Integer(4),
                                typing: None,
                                location: Location::default()
                            },
                            Expression::Constant {
                                constant: Constant::Integer(5),
                                typing: None,
                                location: Location::default()
                            },
                        ],
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Integer(3),
                        typing: None,
                        location: Location::default()
                    },
                ],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: String::from("sqrt"),
                    typing: None,
                    location: Location::default()
                }),
                inputs: vec![Expression::Application {
                    function_expression: Box::new(Expression::Call {
                        id: BinaryOperator::Sub.to_string(),
                        typing: None,
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression::Application {
                            function_expression: Box::new(Expression::Call {
                                id: BinaryOperator::Mul.to_string(),
                                typing: None,
                                location: Location::default()
                            }),
                            inputs: vec![
                                Expression::Constant {
                                    constant: Constant::Integer(4),
                                    typing: None,
                                    location: Location::default()
                                },
                                Expression::Constant {
                                    constant: Constant::Integer(5),
                                    typing: None,
                                    location: Location::default()
                                },
                            ],
                            typing: None,
                            location: Location::default()
                        },
                        Expression::Constant {
                            constant: Constant::Integer(3),
                            typing: None,
                            location: Location::default()
                        },
                    ],
                    typing: None,
                    location: Location::default()
                }],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: OtherOperator::Print.to_string(),
                    typing: None,
                    location: Location::default()
                }),
                inputs: vec![Expression::Constant {
                    constant: Constant::String(String::from("Hello world")),
                    typing: None,
                    location: Location::default()
                }],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Abstraction {
                inputs: vec![String::from("x"), String::from("y")],
                expression: Box::new(Expression::Application {
                    function_expression: Box::new(Expression::Call {
                        id: BinaryOperator::Add.to_string(),
                        typing: None,
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression::Call {
                            id: String::from("x"),
                            typing: None,
                            location: Location::default()
                        },
                        Expression::Call {
                            id: String::from("y"),
                            typing: None,
                            location: Location::default()
                        },
                    ],
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            Expression::TypedAbstraction {
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer)
                ],
                expression: Box::new(Expression::Application {
                    function_expression: Box::new(Expression::Call {
                        id: BinaryOperator::Add.to_string(),
                        typing: None,
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression::Call {
                            id: String::from("x"),
                            typing: None,
                            location: Location::default()
                        },
                        Expression::Call {
                            id: String::from("y"),
                            typing: None,
                            location: Location::default()
                        },
                    ],
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: OtherOperator::IfThenElse.to_string(),
                    typing: None,
                    location: Location::default()
                }),
                inputs: vec![
                    Expression::Call {
                        id: String::from("b"),
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Call {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Call {
                        id: String::from("y"),
                        typing: None,
                        location: Location::default()
                    },
                ],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Structure {
                name: String::from("Point"),
                fields: vec![
                    (
                        String::from("x"),
                        Expression::Constant {
                            constant: Constant::Integer(3),
                            typing: None,
                            location: Location::default()
                        }
                    ),
                    (
                        String::from("y"),
                        Expression::Constant {
                            constant: Constant::Integer(0),
                            typing: None,
                            location: Location::default()
                        }
                    )
                ],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id12, &files.source(file_id12).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Array {
                elements: vec![
                    Expression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Integer(3),
                        typing: None,
                        location: Location::default()
                    }
                ],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id13, &files.source(file_id13).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Array {
                elements: vec![
                    Expression::Constant {
                        constant: Constant::Float(0.01),
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Float(0.01),
                        typing: None,
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Float(0.01),
                        typing: None,
                        location: Location::default()
                    }
                ],
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id14, &files.source(file_id14).unwrap())
            .unwrap();
        assert_eq!(
            Expression::When {
                id: String::from("a"),
                option: Box::new(Expression::Call {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default()
                }),
                present: Box::new(Expression::Call {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                default: Box::new(Expression::Constant {
                    constant: Constant::Integer(0),
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id15, &files.source(file_id15).unwrap())
            .unwrap();
        assert_eq!(
            Expression::When {
                id: String::from("a"),
                option: Box::new(Expression::Call {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                present: Box::new(Expression::Call {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                default: Box::new(Expression::Constant {
                    constant: Constant::Integer(0),
                    typing: None,
                    location: Location::default()
                }),
                typing: None,
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id16, &files.source(file_id16).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Match {
                expression: Box::new(Expression::Call {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default()
                }),
                arms: vec![
                    (
                        Pattern::Structure {
                            name: String::from("Point"),
                            fields: vec![
                                (
                                    String::from("x"),
                                    Pattern::Constant {
                                        constant: Constant::Integer(0),
                                        location: Location::default()
                                    }
                                ),
                                (
                                    String::from("y"),
                                    Pattern::Default {
                                        location: Location::default()
                                    }
                                )
                            ],
                            location: Location::default()
                        },
                        None,
                        Expression::Constant {
                            constant: Constant::Integer(0),
                            typing: None,
                            location: Location::default()
                        }
                    ),
                    (
                        Pattern::Structure {
                            name: String::from("Point"),
                            fields: vec![
                                (
                                    String::from("x"),
                                    Pattern::Identifier {
                                        name: String::from("x"),
                                        location: Location::default()
                                    }
                                ),
                                (
                                    String::from("y"),
                                    Pattern::Default {
                                        location: Location::default()
                                    }
                                )
                            ],
                            location: Location::default()
                        },
                        Some(Expression::Application {
                            function_expression: Box::new(Expression::Call {
                                id: BinaryOperator::Low.to_string(),
                                typing: None,
                                location: Location::default()
                            }),
                            inputs: vec![
                                Expression::Call {
                                    id: String::from("x"),
                                    typing: None,
                                    location: Location::default()
                                },
                                Expression::Constant {
                                    constant: Constant::Integer(0),
                                    typing: None,
                                    location: Location::default()
                                }
                            ],
                            typing: None,
                            location: Location::default()
                        }),
                        Expression::Application {
                            function_expression: Box::new(Expression::Call {
                                id: UnaryOperator::Neg.to_string(),
                                typing: None,
                                location: Location::default()
                            }),
                            inputs: vec![Expression::Constant {
                                constant: Constant::Integer(1),
                                typing: None,
                                location: Location::default()
                            }],
                            typing: None,
                            location: Location::default()
                        }
                    ),
                    (
                        Pattern::Default {
                            location: Location::default()
                        },
                        None,
                        Expression::Constant {
                            constant: Constant::Integer(1),
                            typing: None,
                            location: Location::default()
                        }
                    )
                ],
                typing: None,
                location: Location::default()
            },
            expression
        );
    }

    #[test]
    fn constant() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("unit_test.gr", "()");
        let file_id2 = files.add("bool_test.gr", "true");
        let file_id3 = files.add("int_test.gr", "3");
        let file_id4 = files.add("float_test.gr", "3.540");
        let file_id5 = files.add("string_test.gr", "\"Hello world\"");
        let file_id6 = files.add("enum_test.gr", "Color.Yellow");

        let constant = langrust::constantParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(Constant::Unit, constant);
        let constant = langrust::constantParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(Constant::Boolean(true), constant);
        let constant = langrust::constantParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(Constant::Integer(3), constant);
        let constant = langrust::constantParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(Constant::Float(3.540), constant);
        let constant = langrust::constantParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(Constant::String(String::from("Hello world")), constant);
        let constant = langrust::constantParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            Constant::Enumeration(String::from("Color"), String::from("Yellow")),
            constant
        );
    }
}

#[test]
fn parse_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/counter.gr").expect("unkown file"),
    );

    let file: File = langrust::fileParser::new()
        .parse(counter_id, &files.source(counter_id).unwrap())
        .unwrap();

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/blinking.gr").expect("unkown file"),
    );

    let file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_button_management() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/button_management.gr").expect("unkown file"),
    );

    let file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();

    insta::assert_yaml_snapshot!(file);
}
