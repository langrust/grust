use codespan_reporting::files::SimpleFiles;

use grustine::parsing;

#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::{Files, SimpleFiles};

    use grustine::ast::{
        expression::{Expression, ExpressionKind}, interface::FlowType, pattern::{Pattern, PatternKind}, stream_expression::{StreamExpression, StreamExpressionKind}, typedef::{Typedef, TypedefKind}
    };
    use grustine::common::{
        constant::Constant,
        location::Location,
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        pattern::Pattern,
        r#type::Type,
    };
    use grustine::parser::langrust;

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
            Typedef {
                id: String::from("Point"),
                kind: TypedefKind::Structure {
                    fields: vec![
                        (String::from("x"), Type::Integer),
                        (String::from("y"), Type::Integer),
                    ]
                },
                location: Location::default(),
            }
        );
        let user_type = langrust::userTypeParser::new()
            .parse(enum_test_id, &files.source(enum_test_id).unwrap())
            .unwrap();
        assert_eq!(
            user_type,
            Typedef {
                id: String::from("Color"),
                kind: TypedefKind::Enumeration {
                    elements: vec![
                        String::from("Red"),
                        String::from("Blue"),
                        String::from("Green"),
                        String::from("Yellow"),
                    ]
                },
                location: Location::default(),
            }
        );
        let user_type = langrust::userTypeParser::new()
            .parse(array_test_id, &files.source(array_test_id).unwrap())
            .unwrap();
        assert_eq!(
            user_type,
            Typedef {
                id: String::from("Matrix"),
                kind: TypedefKind::Array {
                    array_type: Type::Array(Box::new(Type::Integer), 3),
                    size: 3
                },
                location: Location::default(),
            }
        );
    }

    #[test]
    fn complete_types() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("int_test.gr", "int");
        let file_id2 = files.add("float_test.gr", "float");
        let file_id3 = files.add("bool_test.gr", "bool");
        let file_id4 = files.add("string_test.gr", "string");
        let file_id5 = files.add("unit_test.gr", "unit");
        let file_id6 = files.add("array_test.gr", "[int; 3]");
        let file_id7 = files.add("option_test.gr", "int?");
        let file_id8 = files.add("undefined_type_test.gr", "Color");
        let file_id9 = files.add("tuple_type_test.gr", "(int, Color)");
        let file_id10 = files.add("function_type_test1.gr", "(int, Color) -> bool");
        let file_id11 = files.add("function_type_test2.gr", "int -> bool");

        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::Integer);
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::Float);
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::Boolean);
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::String);
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::Unit);
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::Array(Box::new(Type::Integer), 3));
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::Option(Box::new(Type::Integer)));
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(complete_type, Type::NotDefinedYet(String::from("Color")));
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            complete_type,
            Type::Tuple(vec![
                Type::Integer,
                Type::NotDefinedYet(String::from("Color"))
            ])
        );
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            complete_type,
            Type::Abstract(
                vec![Type::Integer, Type::NotDefinedYet(String::from("Color"))],
                Box::new(Type::Boolean)
            )
        );
        let complete_type = langrust::completeTypeParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            complete_type,
            Type::Abstract(vec![Type::Integer], Box::new(Type::Boolean))
        );
    }

    #[test]
    fn flow_types() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("int_test.gr", "signal int");
        let file_id2 = files.add("float_test.gr", "signal float");
        let file_id3 = files.add("bool_test.gr", "signal bool");
        let file_id4 = files.add("string_test.gr", "signal string");
        let file_id5 = files.add("unit_test.gr", "signal unit");
        let file_id6 = files.add("array_test.gr", "signal [int; 3]");
        let file_id7 = files.add("option_test.gr", "signal int?");
        let file_id8 = files.add("undefined_type_test.gr", "signal Color");
        let file_id9 = files.add("tuple_type_test.gr", "signal (int, Color)");
        let file_id10 = files.add("function_type_test1.gr", "signal (int, Color) -> bool");
        let file_id11 = files.add("function_type_test2.gr", "signal int -> bool");

        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::Integer));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::Float));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::Boolean));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::String));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::Unit));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::Array(Box::new(Type::Integer), 3)));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::Option(Box::new(Type::Integer))));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(signal_type, FlowType::Signal(Type::NotDefinedYet(String::from("Color"))));
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            signal_type,
            FlowType::Signal(Type::Tuple(vec![
                Type::Integer,
                Type::NotDefinedYet(String::from("Color"))
            ]))
        );
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            signal_type,
            FlowType::Signal(Type::Abstract(
                vec![Type::Integer, Type::NotDefinedYet(String::from("Color"))],
                Box::new(Type::Boolean)
            ))
        );
        let signal_type = langrust::flowTypeParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            signal_type,
            FlowType::Signal(Type::Abstract(vec![Type::Integer], Box::new(Type::Boolean)))
        );
        
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("int_test.gr", "event int");
        let file_id2 = files.add("float_test.gr", "event float");
        let file_id3 = files.add("bool_test.gr", "event bool");
        let file_id4 = files.add("string_test.gr", "event string");
        let file_id5 = files.add("unit_test.gr", "event unit");
        let file_id6 = files.add("array_test.gr", "event [int; 3]");
        let file_id7 = files.add("option_test.gr", "event int?");
        let file_id8 = files.add("undefined_type_test.gr", "event Color");
        let file_id9 = files.add("tuple_type_test.gr", "event (int, Color)");
        let file_id10 = files.add("function_type_test1.gr", "event (int, Color) -> bool");
        let file_id11 = files.add("function_type_test2.gr", "event int -> bool");

        let event_type = langrust::flowTypeParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::Integer));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::Float));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::Boolean));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::String));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::Unit));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::Array(Box::new(Type::Integer), 3)));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::Option(Box::new(Type::Integer))));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(event_type, FlowType::Event(Type::NotDefinedYet(String::from("Color"))));
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            event_type,
            FlowType::Event(Type::Tuple(vec![
                Type::Integer,
                Type::NotDefinedYet(String::from("Color"))
            ]))
        );
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            event_type,
            FlowType::Event(Type::Abstract(
                vec![Type::Integer, Type::NotDefinedYet(String::from("Color"))],
                Box::new(Type::Boolean)
            ))
        );
        let event_type = langrust::flowTypeParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            event_type,
            FlowType::Event(Type::Abstract(vec![Type::Integer], Box::new(Type::Boolean)))
        );
    }

    #[test]
    fn pattern() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("identifier_test.gr", "x");
        let file_id2 = files.add("constant_test.gr", "3");
        let file_id3 = files.add("structure_test.gr", "Point { x: 0, y: _}");
        let file_id4 = files.add("some_test.gr", "some(value)");
        let file_id5 = files.add("none_test.gr", "none");
        let file_id6 = files.add("default_test.gr", "_");
        let file_id7 = files.add("tuple_test.gr", "(x, _)");

        let pattern = langrust::patternParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::Identifier {
                    name: String::from("x")
                },
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::Constant {
                    constant: Constant::Integer(3)
                },
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::Structure {
                    name: String::from("Point"),
                    fields: vec![
                        (
                            String::from("x"),
                            Pattern {
                                kind: PatternKind::Constant {
                                    constant: Constant::Integer(0)
                                },
                                location: Location::default()
                            }
                        ),
                        (
                            String::from("y"),
                            Pattern {
                                kind: PatternKind::Default,
                                location: Location::default()
                            }
                        )
                    ]
                },
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::Some {
                    pattern: Box::new(Pattern {
                        kind: PatternKind::Identifier {
                            name: String::from("value")
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::None,
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::Default,
                location: Location::default()
            },
            pattern
        );
        let pattern = langrust::patternParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            Pattern {
                kind: PatternKind::Tuple {
                    elements: vec![
                        Pattern {
                            kind: PatternKind::Identifier { name: format!("x") },
                            location: Location::default()
                        },
                        Pattern {
                            kind: PatternKind::Default,
                            location: Location::default()
                        }
                    ]
                },
                location: Location::default()
            },
            pattern
        );
    }

    #[test]
    fn stream_expression() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("constant_test.gr", "3");
        let file_id2 = files.add("identifier_test.gr", "x");
        let file_id3 = files.add("brackets_test.gr", "(3)");
        let file_id4 = files.add("unary_test.gr", "-3");
        let file_id5 = files.add("binary_test.gr", "4*5-3");
        let file_id6 = files.add("function_application_test.gr", "sqrt(x*y)");
        let file_id7 = files.add("enumeration_test.gr", "Color::Yellow");
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
            "match a { Point {x: 0, y: _} => 0, Point {x: x, y: _} if x < 0 => -1, _ => 1 }",
        );
        let file_id17 = files.add("field_access_test.gr", "p.x");
        let file_id18 = files.add("map_test.gr", "x.map(f)");
        let file_id19 = files.add("fold_test.gr", "l.fold(0, |sum, x| x + sum)");
        let file_id20 = files.add("sort_test.gr", "l.sort(|a, b| a - b)");
        let file_id21 = files.add("zip_test.gr", "zip(a,b)");
        let file_id22 = files.add("tuple_element_access_test.gr", "my_tuple.0");
        let file_id23 = files.add("tuple.gr", "(0, 1, 2)");

        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Constant {
                        constant: Constant::Integer(3)
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Identifier {
                        id: String::from("x")
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Application {
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: UnaryOperator::Brackets.to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        inputs: vec![StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Constant {
                                    constant: Constant::Integer(3)
                                }
                            },
                            location: Location::default()
                        },]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Application {
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: UnaryOperator::Neg.to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        inputs: vec![StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Constant {
                                    constant: Constant::Integer(3)
                                }
                            },
                            location: Location::default()
                        },]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Application {
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: BinaryOperator::Sub.to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        inputs: vec![
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Application {
                                        function_expression: Box::new(StreamExpression {
                                            kind: StreamExpressionKind::Expression {
                                                expression: ExpressionKind::Identifier {
                                                    id: BinaryOperator::Mul.to_string()
                                                }
                                            },
                                            location: Location::default()
                                        }),
                                        inputs: vec![
                                            StreamExpression {
                                                kind: StreamExpressionKind::Expression {
                                                    expression: ExpressionKind::Constant {
                                                        constant: Constant::Integer(4)
                                                    }
                                                },
                                                location: Location::default()
                                            },
                                            StreamExpression {
                                                kind: StreamExpressionKind::Expression {
                                                    expression: ExpressionKind::Constant {
                                                        constant: Constant::Integer(5)
                                                    }
                                                },
                                                location: Location::default()
                                            },
                                        ]
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(3)
                                    }
                                },
                                location: Location::default()
                            },
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Application {
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("sqrt")
                                }
                            },
                            location: Location::default()
                        }),
                        inputs: vec![StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Application {
                                    function_expression: Box::new(StreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Identifier {
                                                id: BinaryOperator::Mul.to_string()
                                            }
                                        },
                                        location: Location::default()
                                    }),
                                    inputs: vec![
                                        StreamExpression {
                                            kind: StreamExpressionKind::Expression {
                                                expression: ExpressionKind::Identifier {
                                                    id: String::from("x")
                                                }
                                            },
                                            location: Location::default()
                                        },
                                        StreamExpression {
                                            kind: StreamExpressionKind::Expression {
                                                expression: ExpressionKind::Identifier {
                                                    id: String::from("y")
                                                }
                                            },
                                            location: Location::default()
                                        },
                                    ]
                                }
                            },
                            location: Location::default()
                        },]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Enumeration {
                        enum_name: String::from("Color"),
                        elem_name: String::from("Yellow"),
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::FieldAccess {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Application {
                                    function_expression: Box::new(StreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Identifier {
                                                id: String::from("my_node")
                                            }
                                        },
                                        location: Location::default()
                                    }),
                                    inputs: vec![
                                        StreamExpression {
                                            kind: StreamExpressionKind::Expression {
                                                expression: ExpressionKind::Identifier {
                                                    id: String::from("my_input1")
                                                }
                                            },
                                            location: Location::default()
                                        },
                                        StreamExpression {
                                            kind: StreamExpressionKind::Expression {
                                                expression: ExpressionKind::Identifier {
                                                    id: String::from("my_input2")
                                                }
                                            },
                                            location: Location::default()
                                        }
                                    ]
                                }
                            },
                            location: Location::default(),
                        }),
                        field: String::from("my_signal"),
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::FollowedBy {
                    constant: Box::new(StreamExpression {
                        kind: StreamExpressionKind::Expression {
                            expression: ExpressionKind::Constant {
                                constant: Constant::Integer(0)
                            }
                        },
                        location: Location::default()
                    }),
                    expression: Box::new(StreamExpression {
                        kind: StreamExpressionKind::Expression {
                            expression: ExpressionKind::Application {
                                function_expression: Box::new(StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Identifier {
                                            id: BinaryOperator::Add.to_string()
                                        }
                                    },
                                    location: Location::default()
                                }),
                                inputs: vec![
                                    StreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Identifier {
                                                id: String::from("x")
                                            }
                                        },
                                        location: Location::default()
                                    },
                                    StreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Constant {
                                                constant: Constant::Integer(1)
                                            }
                                        },
                                        location: Location::default()
                                    },
                                ]
                            }
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Application {
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: OtherOperator::IfThenElse.to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        inputs: vec![
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier {
                                        id: String::from("b")
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier {
                                        id: String::from("x")
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier {
                                        id: String::from("y")
                                    }
                                },
                                location: Location::default()
                            },
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Constant {
                                            constant: Constant::Integer(3)
                                        }
                                    },
                                    location: Location::default()
                                }
                            ),
                            (
                                String::from("y"),
                                StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Constant {
                                            constant: Constant::Integer(0)
                                        }
                                    },
                                    location: Location::default()
                                }
                            )
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id12, &files.source(file_id12).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Array {
                        elements: vec![
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(1)
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(2)
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(3)
                                    }
                                },
                                location: Location::default()
                            }
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id13, &files.source(file_id13).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Array {
                        elements: vec![
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Float(0.01)
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Float(0.01)
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Float(0.01)
                                    }
                                },
                                location: Location::default()
                            }
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id14, &files.source(file_id14).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::When {
                        id: String::from("a"),
                        option: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("x")
                                }
                            },
                            location: Location::default()
                        }),
                        present: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("a")
                                }
                            },
                            location: Location::default()
                        }),
                        default: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Constant {
                                    constant: Constant::Integer(0)
                                }
                            },
                            location: Location::default()
                        })
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id15, &files.source(file_id15).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::When {
                        id: String::from("a"),
                        option: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("a")
                                }
                            },
                            location: Location::default()
                        }),
                        present: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("a")
                                }
                            },
                            location: Location::default()
                        }),
                        default: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Constant {
                                    constant: Constant::Integer(0)
                                }
                            },
                            location: Location::default()
                        })
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id16, &files.source(file_id16).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Match {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("a")
                                }
                            },
                            location: Location::default()
                        }),
                        arms: vec![
                            (
                                Pattern {
                                    kind: PatternKind::Structure {
                                        name: String::from("Point"),
                                        fields: vec![
                                            (
                                                String::from("x"),
                                                Pattern {
                                                    kind: PatternKind::Constant {
                                                        constant: Constant::Integer(0)
                                                    },
                                                    location: Location::default()
                                                }
                                            ),
                                            (
                                                String::from("y"),
                                                Pattern {
                                                    kind: PatternKind::Default,
                                                    location: Location::default()
                                                }
                                            )
                                        ]
                                    },
                                    location: Location::default()
                                },
                                None,
                                StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Constant {
                                            constant: Constant::Integer(0)
                                        }
                                    },
                                    location: Location::default()
                                }
                            ),
                            (
                                Pattern {
                                    kind: PatternKind::Structure {
                                        name: String::from("Point"),
                                        fields: vec![
                                            (
                                                String::from("x"),
                                                Pattern {
                                                    kind: PatternKind::Identifier {
                                                        name: String::from("x")
                                                    },
                                                    location: Location::default()
                                                }
                                            ),
                                            (
                                                String::from("y"),
                                                Pattern {
                                                    kind: PatternKind::Default,
                                                    location: Location::default()
                                                }
                                            )
                                        ]
                                    },
                                    location: Location::default()
                                },
                                Some(StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Application {
                                            function_expression: Box::new(StreamExpression {
                                                kind: StreamExpressionKind::Expression {
                                                    expression: ExpressionKind::Identifier {
                                                        id: BinaryOperator::Low.to_string()
                                                    }
                                                },
                                                location: Location::default()
                                            }),
                                            inputs: vec![
                                                StreamExpression {
                                                    kind: StreamExpressionKind::Expression {
                                                        expression: ExpressionKind::Identifier {
                                                            id: String::from("x"),
                                                        }
                                                    },
                                                    location: Location::default()
                                                },
                                                StreamExpression {
                                                    kind: StreamExpressionKind::Expression {
                                                        expression: ExpressionKind::Constant {
                                                            constant: Constant::Integer(0),
                                                        }
                                                    },
                                                    location: Location::default()
                                                }
                                            ]
                                        }
                                    },
                                    location: Location::default()
                                }),
                                StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Application {
                                            function_expression: Box::new(StreamExpression {
                                                kind: StreamExpressionKind::Expression {
                                                    expression: ExpressionKind::Identifier {
                                                        id: UnaryOperator::Neg.to_string()
                                                    }
                                                },
                                                location: Location::default()
                                            }),
                                            inputs: vec![StreamExpression {
                                                kind: StreamExpressionKind::Expression {
                                                    expression: ExpressionKind::Constant {
                                                        constant: Constant::Integer(1)
                                                    }
                                                },
                                                location: Location::default()
                                            }]
                                        }
                                    },
                                    location: Location::default()
                                }
                            ),
                            (
                                Pattern {
                                    kind: PatternKind::Default,
                                    location: Location::default()
                                },
                                None,
                                StreamExpression {
                                    kind: StreamExpressionKind::Expression {
                                        expression: ExpressionKind::Constant {
                                            constant: Constant::Integer(1)
                                        }
                                    },
                                    location: Location::default()
                                }
                            )
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id17, &files.source(file_id17).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::FieldAccess {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("p")
                                }
                            },
                            location: Location::default()
                        }),
                        field: "x".to_string()
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id18, &files.source(file_id18).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Map {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("x")
                                }
                            },
                            location: Location::default()
                        }),
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: String::from("f")
                                }
                            },
                            location: Location::default()
                        })
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id19, &files.source(file_id19).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Fold {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: "l".to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        initialization_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Constant {
                                    constant: Constant::Integer(0)
                                }
                            },
                            location: Location::default()
                        }),
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Abstraction {
                                    inputs: vec![String::from("sum"), String::from("x")],
                                    expression: Box::new(StreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Application {
                                                function_expression: Box::new(StreamExpression {
                                                    kind: StreamExpressionKind::Expression {
                                                        expression: ExpressionKind::Identifier {
                                                            id: BinaryOperator::Add.to_string()
                                                        }
                                                    },
                                                    location: Location::default()
                                                }),
                                                inputs: vec![
                                                    StreamExpression {
                                                        kind: StreamExpressionKind::Expression {
                                                            expression:
                                                                ExpressionKind::Identifier {
                                                                    id: String::from("x")
                                                                }
                                                        },
                                                        location: Location::default()
                                                    },
                                                    StreamExpression {
                                                        kind: StreamExpressionKind::Expression {
                                                            expression:
                                                                ExpressionKind::Identifier {
                                                                    id: String::from("sum")
                                                                }
                                                        },
                                                        location: Location::default()
                                                    },
                                                ]
                                            }
                                        },
                                        location: Location::default()
                                    })
                                }
                            },
                            location: Location::default()
                        })
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id20, &files.source(file_id20).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Sort {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: "l".to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        function_expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Abstraction {
                                    inputs: vec![String::from("a"), String::from("b")],
                                    expression: Box::new(StreamExpression {
                                        kind: StreamExpressionKind::Expression {
                                            expression: ExpressionKind::Application {
                                                function_expression: Box::new(StreamExpression {
                                                    kind: StreamExpressionKind::Expression {
                                                        expression: ExpressionKind::Identifier {
                                                            id: BinaryOperator::Sub.to_string()
                                                        }
                                                    },
                                                    location: Location::default()
                                                }),
                                                inputs: vec![
                                                    StreamExpression {
                                                        kind: StreamExpressionKind::Expression {
                                                            expression:
                                                                ExpressionKind::Identifier {
                                                                    id: String::from("a")
                                                                }
                                                        },
                                                        location: Location::default()
                                                    },
                                                    StreamExpression {
                                                        kind: StreamExpressionKind::Expression {
                                                            expression:
                                                                ExpressionKind::Identifier {
                                                                    id: String::from("b")
                                                                }
                                                        },
                                                        location: Location::default()
                                                    },
                                                ]
                                            }
                                        },
                                        location: Location::default()
                                    })
                                }
                            },
                            location: Location::default()
                        })
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id21, &files.source(file_id21).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Zip {
                        arrays: vec![
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier {
                                        id: "a".to_string()
                                    }
                                },
                                location: Location::default()
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier {
                                        id: "b".to_string()
                                    }
                                },
                                location: Location::default()
                            }
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id22, &files.source(file_id22).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::TupleElementAccess {
                        expression: Box::new(StreamExpression {
                            kind: StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier {
                                    id: "my_tuple".to_string()
                                }
                            },
                            location: Location::default()
                        }),
                        element_number: 0
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
        let stream_expression = langrust::streamExpressionParser::new()
            .parse(file_id23, &files.source(file_id23).unwrap())
            .unwrap();
        assert_eq!(
            StreamExpression {
                kind: StreamExpressionKind::Expression {
                    expression: ExpressionKind::Tuple {
                        elements: vec![
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(0)
                                    }
                                },
                                location: Location::default(),
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(1)
                                    }
                                },
                                location: Location::default(),
                            },
                            StreamExpression {
                                kind: StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Constant {
                                        constant: Constant::Integer(2)
                                    }
                                },
                                location: Location::default(),
                            },
                        ]
                    }
                },
                location: Location::default()
            },
            stream_expression
        );
    }

    #[test]
    fn expression() {
        let mut files = SimpleFiles::new();
        let file_id1 = files.add("constant_test.gr", "3");
        let file_id2 = files.add("identifier_test.gr", "x");
        let file_id3 = files.add("brackets_test.gr", "(3)");
        let file_id4 = files.add("unary_test.gr", "-3");
        let file_id5 = files.add("binary_test.gr", "4*5-3");
        let file_id6 = files.add("function_application_test.gr", "sqrt(4*5-3)");
        let file_id7 = files.add("enumeration_test.gr", "Color::Yellow");
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
            "match a { Point {x: 0, y: _} => 0, Point {x: x, y: _} if x < 0 => -1, _ => 1 }",
        );
        let file_id17 = files.add("field_access_test.gr", "p.x");
        let file_id18 = files.add("map_test.gr", "l.map(f)");
        let file_id19 = files.add("fold_test.gr", "l.fold(0, |sum, x| x + sum)");
        let file_id20 = files.add("sort_test.gr", "l.sort(|a, b| a - b)");
        let file_id21 = files.add("zip_test.gr", "zip(a,b)");
        let file_id22 = files.add("tuple_element_access_test.gr", "my_tuple.0");
        let file_id23 = files.add("tuple.gr", "(0, 1, 2)");

        let expression = langrust::expressionParser::new()
            .parse(file_id1, &files.source(file_id1).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Constant {
                    constant: Constant::Integer(3)
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id2, &files.source(file_id2).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Identifier {
                    id: String::from("x")
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id3, &files.source(file_id3).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Application {
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: UnaryOperator::Brackets.to_string()
                        },
                        location: Location::default()
                    }),
                    inputs: vec![Expression {
                        kind: ExpressionKind::Constant {
                            constant: Constant::Integer(3)
                        },
                        location: Location::default()
                    },]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id4, &files.source(file_id4).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Application {
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: UnaryOperator::Neg.to_string()
                        },
                        location: Location::default()
                    }),
                    inputs: vec![Expression {
                        kind: ExpressionKind::Constant {
                            constant: Constant::Integer(3)
                        },
                        location: Location::default()
                    },]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id5, &files.source(file_id5).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Application {
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: BinaryOperator::Sub.to_string()
                        },
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression {
                            kind: ExpressionKind::Application {
                                function_expression: Box::new(Expression {
                                    kind: ExpressionKind::Identifier {
                                        id: BinaryOperator::Mul.to_string()
                                    },
                                    location: Location::default()
                                }),
                                inputs: vec![
                                    Expression {
                                        kind: ExpressionKind::Constant {
                                            constant: Constant::Integer(4)
                                        },
                                        location: Location::default()
                                    },
                                    Expression {
                                        kind: ExpressionKind::Constant {
                                            constant: Constant::Integer(5)
                                        },
                                        location: Location::default()
                                    },
                                ]
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(3)
                            },
                            location: Location::default()
                        },
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id6, &files.source(file_id6).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Application {
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("sqrt")
                        },
                        location: Location::default()
                    }),
                    inputs: vec![Expression {
                        kind: ExpressionKind::Application {
                            function_expression: Box::new(Expression {
                                kind: ExpressionKind::Identifier {
                                    id: BinaryOperator::Sub.to_string()
                                },
                                location: Location::default()
                            }),
                            inputs: vec![
                                Expression {
                                    kind: ExpressionKind::Application {
                                        function_expression: Box::new(Expression {
                                            kind: ExpressionKind::Identifier {
                                                id: BinaryOperator::Mul.to_string()
                                            },
                                            location: Location::default()
                                        }),
                                        inputs: vec![
                                            Expression {
                                                kind: ExpressionKind::Constant {
                                                    constant: Constant::Integer(4),
                                                },
                                                location: Location::default()
                                            },
                                            Expression {
                                                kind: ExpressionKind::Constant {
                                                    constant: Constant::Integer(5),
                                                },
                                                location: Location::default()
                                            },
                                        ]
                                    },
                                    location: Location::default()
                                },
                                Expression {
                                    kind: ExpressionKind::Constant {
                                        constant: Constant::Integer(3)
                                    },
                                    location: Location::default()
                                },
                            ]
                        },
                        location: Location::default()
                    }]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id7, &files.source(file_id7).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Enumeration {
                    enum_name: String::from("Color"),
                    elem_name: String::from("Yellow"),
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id8, &files.source(file_id8).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Abstraction {
                    inputs: vec![String::from("x"), String::from("y")],
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Application {
                            function_expression: Box::new(Expression {
                                kind: ExpressionKind::Identifier {
                                    id: BinaryOperator::Add.to_string()
                                },
                                location: Location::default()
                            }),
                            inputs: vec![
                                Expression {
                                    kind: ExpressionKind::Identifier {
                                        id: String::from("x")
                                    },
                                    location: Location::default()
                                },
                                Expression {
                                    kind: ExpressionKind::Identifier {
                                        id: String::from("y")
                                    },
                                    location: Location::default()
                                },
                            ]
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id9, &files.source(file_id9).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::TypedAbstraction {
                    inputs: vec![
                        (String::from("x"), Type::Integer),
                        (String::from("y"), Type::Integer)
                    ],
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Application {
                            function_expression: Box::new(Expression {
                                kind: ExpressionKind::Identifier {
                                    id: BinaryOperator::Add.to_string()
                                },
                                location: Location::default()
                            }),
                            inputs: vec![
                                Expression {
                                    kind: ExpressionKind::Identifier {
                                        id: String::from("x")
                                    },
                                    location: Location::default()
                                },
                                Expression {
                                    kind: ExpressionKind::Identifier {
                                        id: String::from("y")
                                    },
                                    location: Location::default()
                                },
                            ]
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id10, &files.source(file_id10).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Application {
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: OtherOperator::IfThenElse.to_string()
                        },
                        location: Location::default()
                    }),
                    inputs: vec![
                        Expression {
                            kind: ExpressionKind::Identifier {
                                id: String::from("b")
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Identifier {
                                id: String::from("x")
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Identifier {
                                id: String::from("y")
                            },
                            location: Location::default()
                        },
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id11, &files.source(file_id11).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Structure {
                    name: String::from("Point"),
                    fields: vec![
                        (
                            String::from("x"),
                            Expression {
                                kind: ExpressionKind::Constant {
                                    constant: Constant::Integer(3)
                                },
                                location: Location::default()
                            }
                        ),
                        (
                            String::from("y"),
                            Expression {
                                kind: ExpressionKind::Constant {
                                    constant: Constant::Integer(0)
                                },
                                location: Location::default()
                            }
                        )
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id12, &files.source(file_id12).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Array {
                    elements: vec![
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(1)
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(2)
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(3)
                            },
                            location: Location::default()
                        }
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id13, &files.source(file_id13).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Array {
                    elements: vec![
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Float(0.01)
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Float(0.01)
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Float(0.01)
                            },
                            location: Location::default()
                        }
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id14, &files.source(file_id14).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::When {
                    id: String::from("a"),
                    option: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("x")
                        },
                        location: Location::default()
                    }),
                    present: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("a")
                        },
                        location: Location::default()
                    }),
                    default: Box::new(Expression {
                        kind: ExpressionKind::Constant {
                            constant: Constant::Integer(0)
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id15, &files.source(file_id15).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::When {
                    id: String::from("a"),
                    option: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("a")
                        },
                        location: Location::default()
                    }),
                    present: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("a")
                        },
                        location: Location::default()
                    }),
                    default: Box::new(Expression {
                        kind: ExpressionKind::Constant {
                            constant: Constant::Integer(0)
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id16, &files.source(file_id16).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Match {
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("a")
                        },
                        location: Location::default()
                    }),
                    arms: vec![
                        (
                            Pattern {
                                kind: PatternKind::Structure {
                                    name: String::from("Point"),
                                    fields: vec![
                                        (
                                            String::from("x"),
                                            Pattern {
                                                kind: PatternKind::Constant {
                                                    constant: Constant::Integer(0),
                                                },
                                                location: Location::default()
                                            }
                                        ),
                                        (
                                            String::from("y"),
                                            Pattern {
                                                kind: PatternKind::Default,
                                                location: Location::default()
                                            }
                                        )
                                    ],
                                },
                                location: Location::default()
                            },
                            None,
                            Expression {
                                kind: ExpressionKind::Constant {
                                    constant: Constant::Integer(0)
                                },
                                location: Location::default()
                            }
                        ),
                        (
                            Pattern {
                                kind: PatternKind::Structure {
                                    name: String::from("Point"),
                                    fields: vec![
                                        (
                                            String::from("x"),
                                            Pattern {
                                                kind: PatternKind::Identifier {
                                                    name: String::from("x")
                                                },
                                                location: Location::default()
                                            }
                                        ),
                                        (
                                            String::from("y"),
                                            Pattern {
                                                kind: PatternKind::Default,
                                                location: Location::default()
                                            }
                                        )
                                    ]
                                },
                                location: Location::default()
                            },
                            Some(Expression {
                                kind: ExpressionKind::Application {
                                    function_expression: Box::new(Expression {
                                        kind: ExpressionKind::Identifier {
                                            id: BinaryOperator::Low.to_string()
                                        },
                                        location: Location::default()
                                    }),
                                    inputs: vec![
                                        Expression {
                                            kind: ExpressionKind::Identifier {
                                                id: String::from("x"),
                                            },
                                            location: Location::default()
                                        },
                                        Expression {
                                            kind: ExpressionKind::Constant {
                                                constant: Constant::Integer(0),
                                            },
                                            location: Location::default()
                                        }
                                    ]
                                },
                                location: Location::default()
                            }),
                            Expression {
                                kind: ExpressionKind::Application {
                                    function_expression: Box::new(Expression {
                                        kind: ExpressionKind::Identifier {
                                            id: UnaryOperator::Neg.to_string()
                                        },
                                        location: Location::default()
                                    }),
                                    inputs: vec![Expression {
                                        kind: ExpressionKind::Constant {
                                            constant: Constant::Integer(1)
                                        },
                                        location: Location::default()
                                    }]
                                },
                                location: Location::default()
                            }
                        ),
                        (
                            Pattern {
                                kind: PatternKind::Default,
                                location: Location::default()
                            },
                            None,
                            Expression {
                                kind: ExpressionKind::Constant {
                                    constant: Constant::Integer(1)
                                },
                                location: Location::default()
                            }
                        )
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id17, &files.source(file_id17).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::FieldAccess {
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("p")
                        },
                        location: Location::default()
                    }),
                    field: "x".to_string()
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id18, &files.source(file_id18).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Map {
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("l")
                        },
                        location: Location::default()
                    }),
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: String::from("f")
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id19, &files.source(file_id19).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Fold {
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: "l".to_string()
                        },
                        location: Location::default()
                    }),
                    initialization_expression: Box::new(Expression {
                        kind: ExpressionKind::Constant {
                            constant: Constant::Integer(0)
                        },
                        location: Location::default()
                    }),
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Abstraction {
                            inputs: vec![String::from("sum"), String::from("x")],
                            expression: Box::new(Expression {
                                kind: ExpressionKind::Application {
                                    function_expression: Box::new(Expression {
                                        kind: ExpressionKind::Identifier {
                                            id: BinaryOperator::Add.to_string()
                                        },
                                        location: Location::default()
                                    }),
                                    inputs: vec![
                                        Expression {
                                            kind: ExpressionKind::Identifier {
                                                id: String::from("x")
                                            },
                                            location: Location::default()
                                        },
                                        Expression {
                                            kind: ExpressionKind::Identifier {
                                                id: String::from("sum")
                                            },
                                            location: Location::default()
                                        },
                                    ]
                                },
                                location: Location::default()
                            })
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id20, &files.source(file_id20).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Sort {
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: "l".to_string()
                        },
                        location: Location::default()
                    }),
                    function_expression: Box::new(Expression {
                        kind: ExpressionKind::Abstraction {
                            inputs: vec![String::from("a"), String::from("b")],
                            expression: Box::new(Expression {
                                kind: ExpressionKind::Application {
                                    function_expression: Box::new(Expression {
                                        kind: ExpressionKind::Identifier {
                                            id: BinaryOperator::Sub.to_string()
                                        },
                                        location: Location::default()
                                    }),
                                    inputs: vec![
                                        Expression {
                                            kind: ExpressionKind::Identifier {
                                                id: String::from("a")
                                            },
                                            location: Location::default()
                                        },
                                        Expression {
                                            kind: ExpressionKind::Identifier {
                                                id: String::from("b")
                                            },
                                            location: Location::default()
                                        },
                                    ]
                                },
                                location: Location::default()
                            })
                        },
                        location: Location::default()
                    })
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id21, &files.source(file_id21).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Zip {
                    arrays: vec![
                        Expression {
                            kind: ExpressionKind::Identifier {
                                id: "a".to_string()
                            },
                            location: Location::default()
                        },
                        Expression {
                            kind: ExpressionKind::Identifier {
                                id: "b".to_string()
                            },
                            location: Location::default()
                        }
                    ]
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id22, &files.source(file_id22).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::TupleElementAccess {
                    expression: Box::new(Expression {
                        kind: ExpressionKind::Identifier {
                            id: "my_tuple".to_string()
                        },
                        location: Location::default()
                    }),
                    element_number: 0
                },
                location: Location::default()
            },
            expression
        );
        let expression = langrust::expressionParser::new()
            .parse(file_id23, &files.source(file_id23).unwrap())
            .unwrap();
        assert_eq!(
            Expression {
                kind: ExpressionKind::Tuple {
                    elements: vec![
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(0)
                            },
                            location: Location::default(),
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(1)
                            },
                            location: Location::default(),
                        },
                        Expression {
                            kind: ExpressionKind::Constant {
                                constant: Constant::Integer(2)
                            },
                            location: Location::default(),
                        },
                    ]
                },
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
    }
}

#[test]
fn parse_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/parser/counter.gr").expect("unkown file"),
    );

    let file = parsing(counter_id, &mut files);

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/parser/blinking.gr").expect("unkown file"),
    );

    let file = parsing(blinking_id, &mut files);

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_button_management() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/parser/button_management.gr").expect("unkown file"),
    );

    let file = parsing(blinking_id, &mut files);

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/parser/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    let file = parsing(blinking_id, &mut files);

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/parser/button_management_using_function.gr")
            .expect("unkown file"),
    );

    let file = parsing(blinking_id, &mut files);

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn parse_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/parser/pid.gr").expect("unkown file"),
    );

    let file = parsing(pid_id, &mut files);

    insta::assert_yaml_snapshot!(file);
}
