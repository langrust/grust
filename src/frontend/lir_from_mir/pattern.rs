use crate::ast::pattern::Pattern;
use crate::lir::pattern::{FieldPattern, Pattern as LIRPattern};

/// Transform MIR pattern into LIR pattern.
pub fn lir_from_mir(pattern: Pattern) -> LIRPattern {
    match pattern {
        Pattern::Identifier { name, .. } => LIRPattern::Identifier {
            reference: false,
            mutable: false,
            identifier: name,
        },
        Pattern::Constant { constant, .. } => LIRPattern::Literal { literal: constant },
        Pattern::Structure { name, fields, .. } => {
            let fields = fields
                .into_iter()
                .map(|(name, pattern)| FieldPattern {
                    name,
                    pattern: lir_from_mir(pattern),
                })
                .collect();
            LIRPattern::Structure {
                name,
                fields,
                dots: false,
            }
        }
        Pattern::Some { pattern, .. } => LIRPattern::TupleStructure {
            name: String::from("Some"),
            elements: vec![lir_from_mir(*pattern)],
        },
        Pattern::None { .. } => LIRPattern::Default,
        Pattern::Default { .. } => LIRPattern::Default,
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::ast::pattern::Pattern;
    use crate::common::constant::Constant;
    use crate::common::location::Location;
    use crate::frontend::lir_from_mir::pattern::lir_from_mir;
    use crate::lir::pattern::{FieldPattern, Pattern as LIRPattern};

    #[test]
    fn should_create_a_lir_default_pattern_from_a_mir_default_pattern() {
        let pattern = Pattern::Default {
            location: Location::default(),
        };
        let control = LIRPattern::Default;
        assert_eq!(lir_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_lir_default_pattern_from_a_mir_none_pattern() {
        let pattern = Pattern::None {
            location: Location::default(),
        };
        let control = LIRPattern::Default;
        assert_eq!(lir_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_lir_tuple_structure_pattern_from_a_mir_some_pattern() {
        let pattern = Pattern::Some {
            pattern: Box::new(Pattern::Default {
                location: Location::default(),
            }),
            location: Location::default(),
        };
        let control = LIRPattern::TupleStructure {
            name: String::from("Some"),
            elements: vec![LIRPattern::Default],
        };
        assert_eq!(lir_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_lir_literal_pattern_from_a_mir_constant_pattern() {
        let pattern = Pattern::Constant {
            constant: Constant::Integer(1),
            location: Location::default(),
        };
        let control = LIRPattern::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(lir_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_lir_identifier_pattern_owned_and_immutable_from_a_mir_identifier_pattern() {
        let pattern = Pattern::Identifier {
            name: String::from("x"),
            location: Location::default(),
        };
        let control = LIRPattern::Identifier {
            reference: false,
            mutable: false,
            identifier: String::from("x"),
        };
        assert_eq!(lir_from_mir(pattern), control)
    }

    #[test]
    fn should_create_a_lir_structure_pattern_from_a_mir_structure_pattern() {
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
        let control = LIRPattern::Structure {
            name: String::from("Point"),
            fields: vec![
                FieldPattern {
                    name: String::from("x"),
                    pattern: LIRPattern::Default,
                },
                FieldPattern {
                    name: String::from("y"),
                    pattern: LIRPattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("y"),
                    },
                },
            ],
            dots: false,
        };
        assert_eq!(lir_from_mir(pattern), control)
    }
}
