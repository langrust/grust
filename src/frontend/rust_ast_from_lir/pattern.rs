use crate::ast::pattern::Pattern;
use crate::rust_ast::pattern::{FieldPattern, Pattern as RustASTPattern};

/// Transform LIR pattern into RustAST pattern.
pub fn rust_ast_from_mir(pattern: Pattern) -> RustASTPattern {
    match pattern {
        Pattern::Identifier { name, .. } => RustASTPattern::Identifier {
            reference: false,
            mutable: false,
            identifier: name,
        },
        Pattern::Constant { constant, .. } => RustASTPattern::Literal { literal: constant },
        Pattern::Structure { name, fields, .. } => {
            let fields = fields
                .into_iter()
                .map(|(name, pattern)| FieldPattern {
                    name,
                    pattern: rust_ast_from_mir(pattern),
                })
                .collect();
            RustASTPattern::Structure {
                name,
                fields,
                dots: false,
            }
        }
        Pattern::Some { pattern, .. } => RustASTPattern::TupleStructure {
            name: String::from("Some"),
            elements: vec![rust_ast_from_mir(*pattern)],
        },
        Pattern::None { .. } => RustASTPattern::Default,
        Pattern::Default { .. } => RustASTPattern::Default,
    }
}

#[cfg(test)]
mod rust_ast_from_mir {
    use crate::ast::pattern::Pattern;
    use crate::common::constant::Constant;
    use crate::common::location::Location;
    use crate::frontend::rust_ast_from_lir::pattern::rust_ast_from_mir;
    use crate::rust_ast::pattern::{FieldPattern, Pattern as RustASTPattern};

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_mir_default_pattern() {
        let pattern = Pattern::Default {
            location: Location::default(),
        };
        let control = RustASTPattern::Default;
        assert_eq!(rust_ast_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_mir_none_pattern() {
        let pattern = Pattern::None {
            location: Location::default(),
        };
        let control = RustASTPattern::Default;
        assert_eq!(rust_ast_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_tuple_structure_pattern_from_a_mir_some_pattern() {
        let pattern = Pattern::Some {
            pattern: Box::new(Pattern::Default {
                location: Location::default(),
            }),
            location: Location::default(),
        };
        let control = RustASTPattern::TupleStructure {
            name: String::from("Some"),
            elements: vec![RustASTPattern::Default],
        };
        assert_eq!(rust_ast_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_literal_pattern_from_a_mir_constant_pattern() {
        let pattern = Pattern::Constant {
            constant: Constant::Integer(1),
            location: Location::default(),
        };
        let control = RustASTPattern::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(rust_ast_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_identifier_pattern_owned_and_immutable_from_a_mir_identifier_pattern() {
        let pattern = Pattern::Identifier {
            name: String::from("x"),
            location: Location::default(),
        };
        let control = RustASTPattern::Identifier {
            reference: false,
            mutable: false,
            identifier: String::from("x"),
        };
        assert_eq!(rust_ast_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_structure_pattern_from_a_mir_structure_pattern() {
        let pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Pattern::Default {
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Pattern::Identifier {
                        name: String::from("y"),
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };
        let control = RustASTPattern::Structure {
            name: String::from("Point"),
            fields: vec![
                FieldPattern {
                    name: String::from("x"),
                    pattern: RustASTPattern::Default,
                },
                FieldPattern {
                    name: String::from("y"),
                    pattern: RustASTPattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("y"),
                    },
                },
            ],
            dots: false,
        };
        assert_eq!(rust_ast_from_mir(pattern), control)
    }
}
