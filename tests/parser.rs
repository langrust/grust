#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::Files;
    use grustine::ast::{
        component::Component, file::File, function::Function, node::Node,
        user_defined_type::UserDefinedType,
    };
    use grustine::langrust;
    use grustine::util::{files, location::Location};

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
}
