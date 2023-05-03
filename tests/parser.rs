#[cfg(test)]
mod langrust_ast_constructs {
    use codespan_reporting::files::Files;
    use grustine::ast::component::Component;
    use grustine::langrust;
    use grustine::ast::{
        file::File
    };
    use grustine::util::files;
    use grustine::util::location::Location;

    #[test]
    fn file_parser() {
        let mut files = files::Files::new();

        let module_test_id = files.add(
            "module_test.gr",
            "module"
        ).unwrap();
        let program_test_id = files.add(
            "program_test.gr",
            "component"
        ).unwrap();

        let file = langrust::fileParser::new()
            .parse(module_test_id, &files.source(module_test_id).unwrap())
            .unwrap();
        assert_eq!(file, File::Module{ location: Location::default() },);
        
        let file = langrust::fileParser::new()
            .parse(program_test_id, &files.source(program_test_id).unwrap())
            .unwrap();
        assert_eq!(file, File::Program{ 
            component: Component{
                location: Location::default()
            },
            location: Location::default()
        },);
    }
}
