use crate::common::r#type::Type;
use crate::rust_ast::r#type::Type as RustASTType;

/// Transform LIR type into RustAST type.
pub fn rust_ast_from_lir(r#type: Type) -> RustASTType {
    match r#type {
        Type::Integer => RustASTType::Identifier {
            identifier: String::from("i64"),
        },
        Type::Float => RustASTType::Identifier {
            identifier: String::from("f64"),
        },
        Type::Boolean => RustASTType::Identifier {
            identifier: String::from("bool"),
        },
        Type::String => RustASTType::Identifier {
            identifier: String::from("String"),
        },
        Type::Unit => RustASTType::Identifier {
            identifier: String::from("()"),
        },
        Type::Enumeration(identifier) => RustASTType::Identifier { identifier },
        Type::Structure(identifier) => RustASTType::Identifier { identifier },
        Type::Array(element, size) => RustASTType::Array {
            element: Box::new(rust_ast_from_lir(*element)),
            size,
        },
        Type::Option(element) => RustASTType::Generic {
            generic: Box::new(RustASTType::Identifier {
                identifier: String::from("Option"),
            }),
            arguments: vec![rust_ast_from_lir(*element)],
        },
        Type::Abstract(arguments, output) => {
            let arguments = arguments.into_iter().map(rust_ast_from_lir).collect();
            RustASTType::Closure {
                arguments,
                output: Box::new(rust_ast_from_lir(*output)),
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::common::r#type::Type;
    use crate::frontend::rust_ast_from_lir::r#type::rust_ast_from_lir;
    use crate::rust_ast::r#type::Type as RustASTType;

    #[test]
    fn should_create_rust_ast_owned_i64_from_lir_integer() {
        let r#type = Type::Integer;
        let control = RustASTType::Identifier {
            identifier: String::from("i64"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_f64_from_lir_float() {
        let r#type = Type::Float;
        let control = RustASTType::Identifier {
            identifier: String::from("f64"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_bool_from_lir_boolean() {
        let r#type = Type::Boolean;
        let control = RustASTType::Identifier {
            identifier: String::from("bool"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_string_from_lir_string() {
        let r#type = Type::String;
        let control = RustASTType::Identifier {
            identifier: String::from("String"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_unit_from_lir_unit() {
        let r#type = Type::Unit;
        let control = RustASTType::Identifier {
            identifier: String::from("()"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_structure_from_lir_structure() {
        let r#type = Type::Structure(String::from("Point"));
        let control = RustASTType::Identifier {
            identifier: String::from("Point"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_enumeration_from_lir_enumeration() {
        let r#type = Type::Enumeration(String::from("Color"));
        let control = RustASTType::Identifier {
            identifier: String::from("Color"),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_array_from_lir_array() {
        let r#type = Type::Array(Box::new(Type::Float), 5);
        let control = RustASTType::Array {
            element: Box::new(RustASTType::Identifier {
                identifier: String::from("f64"),
            }),
            size: 5,
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_generic_from_lir_option() {
        let r#type = Type::Option(Box::new(Type::Float));
        let control = RustASTType::Generic {
            generic: Box::new(RustASTType::Identifier {
                identifier: String::from("Option"),
            }),
            arguments: vec![RustASTType::Identifier {
                identifier: String::from("f64"),
            }],
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_closure_from_lir_abstract() {
        let r#type = Type::Abstract(vec![Type::Integer], Box::new(Type::Float));
        let control = RustASTType::Closure {
            arguments: vec![RustASTType::Identifier {
                identifier: String::from("i64"),
            }],
            output: Box::new(RustASTType::Identifier {
                identifier: String::from("f64"),
            }),
        };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }
}
