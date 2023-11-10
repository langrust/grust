use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::rust_ast::item::structure::{Field, Structure};
use crate::lir::item::node_file::input::{Input, InputElement};

/// Transform LIR input into RustAST structure.
pub fn rust_ast_from_lir(input: Input) -> Structure {
    let fields = input
        .elements
        .into_iter()
        .map(|InputElement { identifier, r#type }| Field {
            public_visibility: true,
            name: identifier,
            r#type: type_rust_ast_from_lir(r#type),
        })
        .collect();
    Structure {
        public_visibility: true,
        name: input.node_name + "Input",
        fields,
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::common::r#type::Type;
    use crate::backend::rust_ast_from_lir::item::node_file::input::rust_ast_from_lir;
    use crate::rust_ast::item::structure::{Field, Structure};
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::lir::item::node_file::input::{Input, InputElement};

    #[test]
    fn should_create_rust_ast_structure_from_lir_node_input() {
        let input = Input {
            node_name: format!("Node"),
            elements: vec![InputElement {
                identifier: format!("i"),
                r#type: Type::Integer,
            }],
        };
        let control = Structure {
            public_visibility: true,
            name: format!("NodeInput"),
            fields: vec![Field {
                public_visibility: true,
                name: format!("i"),
                r#type: RustASTType::Identifier {
                    identifier: String::from("i64"),
                },
            }],
        };
        assert_eq!(rust_ast_from_lir(input), control)
    }
}
