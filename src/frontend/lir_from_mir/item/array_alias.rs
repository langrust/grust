use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::rust_ast::item::type_alias::TypeAlias;
use crate::rust_ast::r#type::Type as RustASTType;
use crate::mir::item::array_alias::ArrayAlias;

/// Transform MIR array alias into RustAST type alias.
pub fn lir_from_mir(array_alias: ArrayAlias) -> TypeAlias {
    TypeAlias {
        public_visibility: true,
        name: array_alias.name,
        r#type: RustASTType::Array {
            element: Box::new(type_lir_from_mir(array_alias.array_type)),
            size: array_alias.size,
        },
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::item::array_alias::lir_from_mir;
    use crate::rust_ast::item::type_alias::TypeAlias;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::mir::item::array_alias::ArrayAlias;

    #[test]
    fn should_create_lir_type_alias_from_mir_array_alias() {
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
        assert_eq!(lir_from_mir(array_alias), control)
    }
}
