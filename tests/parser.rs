#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::Files;
    use grustine::ast::{
        component::Component, expression::Expression, file::File, function::Function, node::Node,
        stream_expression::StreamExpression, user_defined_type::UserDefinedType,
    };
    use grustine::langrust;
    use grustine::util::{
        constant::Constant,
        files,
        location::Location,
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        type_system::Type,
    };

    #[test]
    fn file_parser() {
        let mut files = files::Files::new();

        let module_test_id = files
            .add("module_test.gr", "function node enum node function node")
            .unwrap();
        let program_test_id = files
            .add(
                "program_test.gr",
                "node component array node function struct function",
            )
            .unwrap();

        let file = langrust::fileParser::new()
            .parse(module_test_id, &files.source(module_test_id).unwrap())
            .unwrap();
        assert_eq!(
            file,
            File::Module {
                user_defined_types: vec![UserDefinedType::Enumeration {
                    location: Location::default()
                }],
                functions: vec![
                    Function {
                        location: Location::default()
                    },
                    Function {
                        location: Location::default()
                    }
                ],
                nodes: vec![
                    Node {
                        location: Location::default()
                    },
                    Node {
                        location: Location::default()
                    },
                    Node {
                        location: Location::default()
                    }
                ],
                location: Location::default()
            },
        );

        let file = langrust::fileParser::new()
            .parse(program_test_id, &files.source(program_test_id).unwrap())
            .unwrap();
        assert_eq!(
            file,
            File::Program {
                user_defined_types: vec![
                    UserDefinedType::Array {
                        location: Location::default()
                    },
                    UserDefinedType::Structure {
                        location: Location::default()
                    }
                ],
                functions: vec![
                    Function {
                        location: Location::default()
                    },
                    Function {
                        location: Location::default()
                    }
                ],
                nodes: vec![
                    Node {
                        location: Location::default()
                    },
                    Node {
                        location: Location::default()
                    }
                ],
                component: Component {
                    location: Location::default()
                },
                location: Location::default()
            },
        );
    }

    #[test]
    fn types() {
        let mut files = files::Files::new();
        let file_id1 = files.add("int_test.gr", "int").unwrap();
        let file_id2 = files.add("float_test.gr", "float").unwrap();
        let file_id3 = files.add("bool_test.gr", "bool").unwrap();
        let file_id4 = files.add("string_test.gr", "string").unwrap();
        let file_id5 = files.add("unit_test.gr", "unit").unwrap();
        let file_id6 = files.add("array_test.gr", "[int; 3]").unwrap();
        let file_id7 = files.add("option_test.gr", "int?").unwrap();
        let file_id8 = files.add("undefined_type_test.gr", "Color").unwrap();

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
    fn stream_expression() {
        let mut files = files::Files::new();
        let file_id1 = files.add("constant_test.gr", "Color.Yellow").unwrap();
        let file_id2 = files.add("signal_call_test.gr", "x").unwrap();
        let file_id3 = files.add("brackets_test.gr", "(3)").unwrap();
        let file_id4 = files.add("unary_test.gr", "-3").unwrap();
        let file_id5 = files.add("binary_test.gr", "4*5-3").unwrap();
        let file_id6 = files
            .add("map_application_test.gr", "(x*y).map(sqrt)")
            .unwrap();
        let file_id7 = files
            .add("print_test.gr", "print(\"Hello world\")")
            .unwrap();
        let file_id8 = files
            .add(
                "node_application_test.gr",
                "my_node(my_input1, my_input2).my_signal",
            )
            .unwrap();
        let file_id9 = files.add("fby_test.gr", "0 fby x + 1").unwrap();
        let file_id10 = files
            .add("ifthenelse_test.gr", "if b then x else y")
            .unwrap();
        let file_id11 = files
            .add("struct_test.gr", "Point { x: 3, y: 0, }")
            .unwrap();
        let file_id12 = files.add("array_test.gr", "[1, 2, 3]").unwrap();
        let file_id13 = files.add("unified_array_test.gr", "[0.01; 3]").unwrap();
        let file_id14 = files
            .add("when_id_test.gr", "when a = x then a else 0")
            .unwrap();
        let file_id15 = files.add("when_test.gr", "when a then a else 0").unwrap();

        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::Constant {
                constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
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
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                expression: Expression::Call {
                    id: UnaryOperator::Brackets.to_string(),
                    location: Location::default()
                },
                inputs: vec![StreamExpression::Constant {
                    constant: Constant::Integer(3),
                    location: Location::default()
                },],
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                expression: Expression::Call {
                    id: UnaryOperator::Neg.to_string(),
                    location: Location::default()
                },
                inputs: vec![StreamExpression::Constant {
                    constant: Constant::Integer(3),
                    location: Location::default()
                },],
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                expression: Expression::Call {
                    id: BinaryOperator::Sub.to_string(),
                    location: Location::default()
                },
                inputs: vec![
                    StreamExpression::MapApplication {
                        expression: Expression::Call {
                            id: BinaryOperator::Mul.to_string(),
                            location: Location::default()
                        },
                        inputs: vec![
                            StreamExpression::Constant {
                                constant: Constant::Integer(4),
                                location: Location::default()
                            },
                            StreamExpression::Constant {
                                constant: Constant::Integer(5),
                                location: Location::default()
                            },
                        ],
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(3),
                        location: Location::default()
                    },
                ],
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                expression: Expression::Call {
                    id: String::from("sqrt"),
                    location: Location::default()
                },
                inputs: vec![StreamExpression::MapApplication {
                    expression: Expression::Call {
                        id: BinaryOperator::Mul.to_string(),
                        location: Location::default()
                    },
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            location: Location::default()
                        },
                        StreamExpression::SignalCall {
                            id: String::from("y"),
                            location: Location::default()
                        },
                    ],
                    location: Location::default()
                },],
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                expression: Expression::Call {
                    id: OtherOperator::Print.to_string(),
                    location: Location::default()
                },
                inputs: vec![StreamExpression::Constant {
                    constant: Constant::String(String::from("Hello world")),
                    location: Location::default()
                }],
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
                        location: Location::default()
                    },
                    StreamExpression::SignalCall {
                        id: String::from("my_input2"),
                        location: Location::default()
                    }
                ],
                signal: String::from("my_signal"),
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
                    expression: Expression::Call {
                        id: BinaryOperator::Add.to_string(),
                        location: Location::default()
                    },
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            location: Location::default()
                        },
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            location: Location::default()
                        },
                    ],
                    location: Location::default()
                }),
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression::MapApplication {
                expression: Expression::Call {
                    id: OtherOperator::IfThenElse.to_string(),
                    location: Location::default()
                },
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: String::from("b"),
                        location: Location::default()
                    },
                    StreamExpression::SignalCall {
                        id: String::from("x"),
                        location: Location::default()
                    },
                    StreamExpression::SignalCall {
                        id: String::from("y"),
                        location: Location::default()
                    },
                ],
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
                            location: Location::default()
                        }
                    ),
                    (
                        String::from("y"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(0),
                            location: Location::default()
                        }
                    )
                ],
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
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Integer(3),
                        location: Location::default()
                    }
                ],
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
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Float(0.01),
                        location: Location::default()
                    },
                    StreamExpression::Constant {
                        constant: Constant::Float(0.01),
                        location: Location::default()
                    }
                ],
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
                    location: Location::default()
                }),
                present: Box::new(StreamExpression::SignalCall {
                    id: String::from("a"),
                    location: Location::default()
                }),
                default: Box::new(StreamExpression::Constant {
                    constant: Constant::Integer(0),
                    location: Location::default()
                }),
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
                    location: Location::default()
                }),
                present: Box::new(StreamExpression::SignalCall {
                    id: String::from("a"),
                    location: Location::default()
                }),
                default: Box::new(StreamExpression::Constant {
                    constant: Constant::Integer(0),
                    location: Location::default()
                }),
                location: Location::default()
            },
            stream_expression
        );
    }

    #[test]
    fn expression() {
        let mut files = files::Files::new();
        let file_id1 = files.add("constant_test.gr", "Color.Yellow").unwrap();
        let file_id2 = files.add("element_call_test.gr", "x").unwrap();
        let file_id3 = files.add("brackets_test.gr", "(3)").unwrap();
        let file_id4 = files.add("unary_test.gr", "-3").unwrap();
        let file_id5 = files.add("binary_test.gr", "4*5-3").unwrap();
        let file_id6 = files
            .add("function_application_test.gr", "sqrt(4*5-3)")
            .unwrap();
        let file_id7 = files
            .add("print_test.gr", "print(\"Hello world\")")
            .unwrap();
        let file_id8 = files.add("abstraction_test.gr", "|x, y| x + y").unwrap();
        let file_id9 = files
            .add("typed_abstraction_test.gr", "|x: int, y: int| x + y")
            .unwrap();
        let file_id10 = files
            .add("ifthenelse_test.gr", "if b then x else y")
            .unwrap();
        let file_id11 = files
            .add("struct_test.gr", "Point { x: 3, y: 0, }")
            .unwrap();
        let file_id12 = files.add("array_test.gr", "[1, 2, 3]").unwrap();
        let file_id13 = files.add("unified_array_test.gr", "[0.01; 3]").unwrap();
        let file_id14 = files
            .add("when_id_test.gr", "when a = x then a else 0")
            .unwrap();
        let file_id15 = files.add("when_test.gr", "when a then a else 0").unwrap();

        let expression = langrust::expressionParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Constant {
                constant: Constant::Enumeration(String::from("Color"), String::from("Yellow")),
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
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                expression: Box::new(Expression::Call {
                    id: UnaryOperator::Brackets.to_string(),
                    location: Location::default()
                }),
                inputs: vec![Expression::Constant {
                    constant: Constant::Integer(3),
                    location: Location::default()
                },],
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                expression: Box::new(Expression::Call {
                    id: UnaryOperator::Neg.to_string(),
                    location: Location::default()
                }),
                inputs: vec![Expression::Constant {
                    constant: Constant::Integer(3),
                    location: Location::default()
                },],
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                expression: Box::new(Expression::Call {
                    id: BinaryOperator::Sub.to_string(),
                    location: Location::default()
                }),
                inputs: vec![
                    Expression::Application {
                        expression: Box::new(Expression::Call {
                            id: BinaryOperator::Mul.to_string(),
                            location: Location::default()
                        }),
                        inputs: vec![
                            Expression::Constant {
                                constant: Constant::Integer(4),
                                location: Location::default()
                            },
                            Expression::Constant {
                                constant: Constant::Integer(5),
                                location: Location::default()
                            },
                        ],
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Integer(3),
                        location: Location::default()
                    },
                ],
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                expression: Box::new(Expression::Call {
                    id: String::from("sqrt"),
                    location: Location::default()
                }),
                inputs: vec![Expression::Application {
                    expression: Box::new(Expression::Call {
                        id: BinaryOperator::Sub.to_string(),
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression::Application {
                            expression: Box::new(Expression::Call {
                                id: BinaryOperator::Mul.to_string(),
                                location: Location::default()
                            }),
                            inputs: vec![
                                Expression::Constant {
                                    constant: Constant::Integer(4),
                                    location: Location::default()
                                },
                                Expression::Constant {
                                    constant: Constant::Integer(5),
                                    location: Location::default()
                                },
                            ],
                            location: Location::default()
                        },
                        Expression::Constant {
                            constant: Constant::Integer(3),
                            location: Location::default()
                        },
                    ],
                    location: Location::default()
                }],
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                expression: Box::new(Expression::Call {
                    id: OtherOperator::Print.to_string(),
                    location: Location::default()
                }),
                inputs: vec![Expression::Constant {
                    constant: Constant::String(String::from("Hello world")),
                    location: Location::default()
                }],
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
                    expression: Box::new(Expression::Call {
                        id: BinaryOperator::Add.to_string(),
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression::Call {
                            id: String::from("x"),
                            location: Location::default()
                        },
                        Expression::Call {
                            id: String::from("y"),
                            location: Location::default()
                        },
                    ],
                    location: Location::default()
                }),
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
                    expression: Box::new(Expression::Call {
                        id: BinaryOperator::Add.to_string(),
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression::Call {
                            id: String::from("x"),
                            location: Location::default()
                        },
                        Expression::Call {
                            id: String::from("y"),
                            location: Location::default()
                        },
                    ],
                    location: Location::default()
                }),
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            Expression::Application {
                expression: Box::new(Expression::Call {
                    id: OtherOperator::IfThenElse.to_string(),
                    location: Location::default()
                }),
                inputs: vec![
                    Expression::Call {
                        id: String::from("b"),
                        location: Location::default()
                    },
                    Expression::Call {
                        id: String::from("x"),
                        location: Location::default()
                    },
                    Expression::Call {
                        id: String::from("y"),
                        location: Location::default()
                    },
                ],
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
                            location: Location::default()
                        }
                    ),
                    (
                        String::from("y"),
                        Expression::Constant {
                            constant: Constant::Integer(0),
                            location: Location::default()
                        }
                    )
                ],
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
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Integer(2),
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Integer(3),
                        location: Location::default()
                    }
                ],
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
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Float(0.01),
                        location: Location::default()
                    },
                    Expression::Constant {
                        constant: Constant::Float(0.01),
                        location: Location::default()
                    }
                ],
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
                    location: Location::default()
                }),
                present: Box::new(Expression::Call {
                    id: String::from("a"),
                    location: Location::default()
                }),
                default: Box::new(Expression::Constant {
                    constant: Constant::Integer(0),
                    location: Location::default()
                }),
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
                    location: Location::default()
                }),
                present: Box::new(Expression::Call {
                    id: String::from("a"),
                    location: Location::default()
                }),
                default: Box::new(Expression::Constant {
                    constant: Constant::Integer(0),
                    location: Location::default()
                }),
                location: Location::default()
            },
            expression
        );
    }

    #[test]
    fn constant() {
        let mut files = files::Files::new();
        let file_id1 = files.add("unit_test.gr", "()").unwrap();
        let file_id2 = files.add("bool_test.gr", "true").unwrap();
        let file_id3 = files.add("int_test.gr", "3").unwrap();
        let file_id4 = files.add("float_test.gr", "3.540").unwrap();
        let file_id5 = files.add("string_test.gr", "\"Hello world\"").unwrap();
        let file_id6 = files.add("enum_test.gr", "Color.Yellow").unwrap();

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
