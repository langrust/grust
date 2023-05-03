#[cfg(test)]
mod langrust_ast_constructs {
    use grustine::langrust;
    use grustine::ast::{
        file::File
    };

    #[test]
    fn file_parser() {
        let file = langrust::fileParser::new()
            .parse("module")
            .unwrap();
        assert_eq!(file, File::Module());
        
        let file = langrust::fileParser::new()
            .parse("program")
            .unwrap();
        assert_eq!(file, File::Program());
    }
}
