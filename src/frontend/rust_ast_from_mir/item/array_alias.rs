use crate::frontend::rust_ast_from_mir::r#type::rust_ast_from_mir as type_rust_ast_from_mir;
use crate::rust_ast::item::type_alias::TypeAlias;
use crate::rust_ast::r#type::Type as RustASTType;
use crate::mir::item::array_alias::ArrayAlias;

/// Transform MIR array alias into RustAST type alias.
pub fn rust_ast_from_mir(array_alias: ArrayAlias) -> TypeAlias {
    TypeAlias {
        public_visibility: true,
        name: array_alias.name,
        r#type: RustASTType::Array {
            element: Box::new(type_rust_ast_from_mir(array_alias.array_type)),
            size: array_alias.size,
        },
    }
}

#[cfg(test)]
mod rust_ast_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::rust_ast_from_mir::item::array_alias::rust_ast_from_mir;
    use crate::rust_ast::item::type_alias::TypeAlias;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::mir::item::array_alias::ArrayAlias;

    #[test]
    fn should_create_rust_ast_type_alias_from_mir_array_alias() {
        let array_alias = ArrayAlias {
            name: String::from("Matrix5x5"),
            array_type: Type::Array(Box::new(Type::Integer), 5),
            size: 5,
        };
        let control = TypeAlias {
            public_visibility: true,
            name: String::from("Matrix5x5"),
            r#type: RustASTType::Array {
                element: Box::new(RustASTType::Array {
                    element: Box::new(RustASTType::Identifier {
                        identifier: String::from("i64"),
                    }),
                    size: 5,
                }),
                size: 5,
            },
        };
        assert_eq!(rust_ast_from_mir(array_alias), control)
    }
}
