use crate::common::r#type::Type;
use crate::rust_ast::r#type::Type as LIRType;

/// Transform MIR type into LIR type.
pub fn lir_from_mir(r#type: Type) -> LIRType {
    match r#type {
        Type::Integer => LIRType::Identifier {
            identifier: String::from("i64"),
        },
        Type::Float => LIRType::Identifier {
            identifier: String::from("f64"),
        },
        Type::Boolean => LIRType::Identifier {
            identifier: String::from("bool"),
        },
        Type::String => LIRType::Identifier {
            identifier: String::from("String"),
        },
        Type::Unit => LIRType::Identifier {
            identifier: String::from("()"),
        },
        Type::Enumeration(identifier) => LIRType::Identifier { identifier },
        Type::Structure(identifier) => LIRType::Identifier { identifier },
        Type::Array(element, size) => LIRType::Array {
            element: Box::new(lir_from_mir(*element)),
            size,
        },
        Type::Option(element) => LIRType::Generic {
            generic: Box::new(LIRType::Identifier {
                identifier: String::from("Option"),
            }),
            arguments: vec![lir_from_mir(*element)],
        },
        Type::Abstract(arguments, output) => {
            let arguments = arguments.into_iter().map(lir_from_mir).collect();
            LIRType::Closure {
                arguments,
                output: Box::new(lir_from_mir(*output)),
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::r#type::lir_from_mir;
    use crate::rust_ast::r#type::Type as LIRType;

    #[test]
    fn should_create_lir_owned_i64_from_mir_integer() {
        let r#type = Type::Integer;
        let control = LIRType::Identifier {
            identifier: String::from("i64"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_f64_from_mir_float() {
        let r#type = Type::Float;
        let control = LIRType::Identifier {
            identifier: String::from("f64"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_bool_from_mir_boolean() {
        let r#type = Type::Boolean;
        let control = LIRType::Identifier {
            identifier: String::from("bool"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_string_from_mir_string() {
        let r#type = Type::String;
        let control = LIRType::Identifier {
            identifier: String::from("String"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_unit_from_mir_unit() {
        let r#type = Type::Unit;
        let control = LIRType::Identifier {
            identifier: String::from("()"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_structure_from_mir_structure() {
        let r#type = Type::Structure(String::from("Point"));
        let control = LIRType::Identifier {
            identifier: String::from("Point"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_enumeration_from_mir_enumeration() {
        let r#type = Type::Enumeration(String::from("Color"));
        let control = LIRType::Identifier {
            identifier: String::from("Color"),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_array_from_mir_array() {
        let r#type = Type::Array(Box::new(Type::Float), 5);
        let control = LIRType::Array {
            element: Box::new(LIRType::Identifier {
                identifier: String::from("f64"),
            }),
            size: 5,
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_generic_from_mir_option() {
        let r#type = Type::Option(Box::new(Type::Float));
        let control = LIRType::Generic {
            generic: Box::new(LIRType::Identifier {
                identifier: String::from("Option"),
            }),
            arguments: vec![LIRType::Identifier {
                identifier: String::from("f64"),
            }],
        };
        assert_eq!(lir_from_mir(r#type), control)
    }

    #[test]
    fn should_create_lir_owned_closure_from_mir_abstract() {
        let r#type = Type::Abstract(vec![Type::Integer], Box::new(Type::Float));
        let control = LIRType::Closure {
            arguments: vec![LIRType::Identifier {
                identifier: String::from("i64"),
            }],
            output: Box::new(LIRType::Identifier {
                identifier: String::from("f64"),
            }),
        };
        assert_eq!(lir_from_mir(r#type), control)
    }
}
