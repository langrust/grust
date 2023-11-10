use itertools::Itertools;

use crate::{ast::pattern::Pattern, lir::item::node_file::import::Import};

impl Pattern {
    /// Get imports induced by pattern.
    pub fn get_imports(&self) -> Vec<Import> {
        match self {
            Pattern::Constant { constant, .. } => constant.get_imports(),
            Pattern::Structure { name, fields, .. } => {
                let mut imports = fields
                    .iter()
                    .flat_map(|(_, pattern)| pattern.get_imports())
                    .collect::<Vec<_>>();
                imports.push(Import::Structure(name.clone()));
                imports.into_iter().unique().collect()
            }
            Pattern::Some { pattern, .. } => pattern.get_imports(),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod get_imports {
    use crate::{
        ast::pattern::Pattern,
        common::{constant::Constant, location::Location},
        lir::item::node_file::import::Import,
    };

    #[test]
    fn should_get_enumeration_import_from_constant_enumeration_pattern() {
        let pattern = Pattern::Constant {
            constant: Constant::Enumeration(format!("Color"), format!("Blue")),
            location: Location::default(),
        };
        let control = vec![Import::Enumeration(format!("Color"))];
        assert_eq!(pattern.get_imports(), control)
    }

    #[test]
    fn should_get_structure_import_from_structure_pattern() {
        let pattern = Pattern::Structure {
            name: format!("Point"),
            fields: vec![
                (
                    format!("x"),
                    Pattern::Identifier {
                        name: format!("x"),
                        location: Location::default(),
                    },
                ),
                (
                    format!("y"),
                    Pattern::Default {
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };
        let control = vec![Import::Structure(format!("Point"))];
        assert_eq!(pattern.get_imports(), control)
    }

    #[test]
    fn should_not_duplicate_imports() {
        let pattern = Pattern::Structure {
            name: format!("Foo"),
            fields: vec![
                (
                    format!("x"),
                    Pattern::Structure {
                        name: format!("Foo"),
                        fields: vec![
                            (
                                format!("x"),
                                Pattern::Identifier {
                                    name: format!("x"),
                                    location: Location::default(),
                                },
                            ),
                            (
                                format!("y"),
                                Pattern::Constant {
                                    constant: Constant::Enumeration(
                                        format!("Color"),
                                        format!("Blue"),
                                    ),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                ),
                (
                    format!("y"),
                    Pattern::Constant {
                        constant: Constant::Enumeration(format!("Color"), format!("Blue")),
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };
        let control = vec![
            Import::Enumeration(format!("Color")),
            Import::Structure(format!("Foo")),
        ];
        assert_eq!(pattern.get_imports(), control)
    }
}
