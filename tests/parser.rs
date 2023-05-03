#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::Files;
    use grustine::ast::{
        component::Component, file::File, function::Function, node::Node,
        user_defined_type::UserDefinedType,
    };
    use grustine::langrust;
    use grustine::util::{files, location::Location, type_system::Type};

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
}
