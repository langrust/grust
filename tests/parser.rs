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
        operator::{BinaryOperator, UnaryOperator},
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
        let stream_expression = langrust::expressionParser::new()
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
            stream_expression
        );
        let stream_expression = langrust::expressionParser::new()
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
            stream_expression
        );
        let stream_expression = langrust::expressionParser::new()
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
            stream_expression
        );
        let stream_expression = langrust::expressionParser::new()
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
            stream_expression
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
