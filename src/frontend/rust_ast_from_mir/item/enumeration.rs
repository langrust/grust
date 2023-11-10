use crate::rust_ast::item::enumeration::Enumeration as RustASTEnumeration;
use crate::mir::item::enumeration::Enumeration;

/// Transform MIR enumeration into RustAST enumeration.
pub fn rust_ast_from_mir(enumeration: Enumeration) -> RustASTEnumeration {
    RustASTEnumeration {
        public_visibility: true,
        name: enumeration.name,
        elements: enumeration.elements,
    }
}

#[cfg(test)]
mod rust_ast_from_mir {
    use crate::frontend::rust_ast_from_mir::item::enumeration::rust_ast_from_mir;
    use crate::rust_ast::item::enumeration::Enumeration as RustASTEnumeration;
    use crate::mir::item::enumeration::Enumeration;

    #[test]
    fn should_create_rust_ast_enumeration_from_mir_enumeration() {
        let enumeration = Enumeration {
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Green"),
            ],
        };
        let control = RustASTEnumeration {
            public_visibility: true,
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Green"),
            ],
        };
        assert_eq!(rust_ast_from_mir(enumeration), control)
    }
}
