#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::Files;
    use grustine::ast::{component::Component, file::File, function::Function, node::Node};
    use grustine::langrust;
    use grustine::util::{files, location::Location};
    
    #[test]
    fn file_parser() {
        let mut files = files::Files::new();

        let module_test_id = files
            .add("module_test.gr", "function node node function node")
            .unwrap();
        let program_test_id = files
            .add("program_test.gr", "node component node function function")
            .unwrap();

        let file = langrust::fileParser::new()
            .parse(module_test_id, &files.source(module_test_id).unwrap())
            .unwrap();
        assert_eq!(
            file,
            File::Module {
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
