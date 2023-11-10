use crate::frontend::rust_ast_from_mir::r#type::rust_ast_from_mir as type_rust_ast_from_mir;
use crate::rust_ast::item::structure::{Field, Structure as RustASTStructure};
use crate::lir::item::structure::Structure;

/// Transform LIR structure into RustAST structure.
pub fn rust_ast_from_mir(structure: Structure) -> RustASTStructure {
    let fields = structure
        .fields
        .into_iter()
        .map(|(name, r#type)| Field {
            public_visibility: true,
            name,
            r#type: type_rust_ast_from_mir(r#type),
        })
        .collect();
    RustASTStructure {
        public_visibility: true,
        name: structure.name,
        fields,
    }
}

#[cfg(test)]
mod rust_ast_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::rust_ast_from_mir::item::structure::rust_ast_from_mir;
    use crate::rust_ast::item::structure::{Field, Structure as RustASTStructure};
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::lir::item::structure::Structure;

    #[test]
    fn should_create_rust_ast_structure_from_mir_structure() {
        let structure = Structure {
            name: String::from("Point"),
            fields: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
        };
        let control = RustASTStructure {
            public_visibility: true,
            name: String::from("Point"),
            fields: vec![
                Field {
                    public_visibility: true,
                    name: String::from("x"),
                    r#type: RustASTType::Identifier {
                        identifier: String::from("i64"),
                    },
                },
                Field {
                    public_visibility: true,
                    name: String::from("y"),
                    r#type: RustASTType::Identifier {
                        identifier: String::from("i64"),
                    },
                },
            ],
        };
        assert_eq!(rust_ast_from_mir(structure), control)
    }
}
