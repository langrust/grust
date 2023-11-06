/// An import declaration.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum Import {
    /// A module import: `mod my_submodule`.
    Module {
        /// Visibility: `true` is public, `false` is private.
        public_visibility: bool,
        /// Module's name.
        name: String,
    },
    /// An `use` import: `use std::sync::{Arc, Mutex};`
    Use {
        /// Visibility: `true` is public, `false` is private.
        public_visibility: bool,
        /// The path tree.
        tree: PathTree,
    },
}

impl std::fmt::Display for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Import::Module {
                public_visibility,
                name,
            } => {
                let visibility = if *public_visibility { "pub " } else { "" };
                write!(f, "{}mod {};", visibility, name)
            }
            Import::Use {
                public_visibility,
                tree,
            } => {
                let visibility = if *public_visibility { "pub " } else { "" };
                write!(f, "{}use {};", visibility, tree)
            }
        }
    }
}

/// A path of an `use` import.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum PathTree {
    /// Path prefix of import: `std::sync::...`
    Path {
        /// Prefix of the path, corresponding to a module.
        module_name: String,
        /// Next tree.
        tree: Box<PathTree>,
    },
    /// Specific item that can be aliased: `std::sync::Arc as AliasedArc`
    Name {
        /// Import name.
        name: String,
        /// Optional alias for the import.
        alias: Option<String>,
    },
    /// Bunch of items from a module: `std::sync::{Arc, Mutex}`
    Group {
        /// Grouped imports.
        trees: Vec<PathTree>,
    },
    /// All items from module: `std::*`
    Star,
}

impl std::fmt::Display for PathTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathTree::Path { module_name, tree } => write!(f, "{module_name}::{tree}"),
            PathTree::Name { name, alias } => {
                let alias = if let Some(alias) = alias {
                    format!(" as {alias}")
                } else {
                    "".to_string()
                };
                write!(f, "{name}{alias}")
            }
            PathTree::Group { trees } => {
                let trees = trees
                    .iter()
                    .map(|path| format!("{path}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{ {} }}", trees)
            }
            PathTree::Star => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod fmt {
    use crate::lir::item::import::{Import, PathTree};

    #[test]
    fn should_format_use_import_definition_with_name_path() {
        let use_import = Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("fmt"),
                    tree: Box::new(PathTree::Name {
                        name: String::from("Debug"),
                        alias: None,
                    }),
                }),
            },
        };
        let control = String::from("pub use std::fmt::Debug;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_alias_name_path() {
        let use_import = Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("future"),
                    tree: Box::new(PathTree::Name {
                        name: String::from("Future"),
                        alias: Some(String::from("AliasFuture")),
                    }),
                }),
            },
        };
        let control = String::from("pub use std::future::Future as AliasFuture;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_star_path() {
        let use_import = Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("sync"),
                    tree: Box::new(PathTree::Star),
                }),
            },
        };
        let control = String::from("pub use std::sync::*;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_group_path() {
        let use_import = Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Group {
                    trees: vec![
                        PathTree::Path {
                            module_name: String::from("sync"),
                            tree: Box::new(PathTree::Star),
                        },
                        PathTree::Path {
                            module_name: String::from("fmt"),
                            tree: Box::new(PathTree::Name {
                                name: String::from("Debug"),
                                alias: None,
                            }),
                        },
                        PathTree::Path {
                            module_name: String::from("future"),
                            tree: Box::new(PathTree::Name {
                                name: String::from("Future"),
                                alias: Some(String::from("AliasFuture")),
                            }),
                        },
                    ],
                }),
            },
        };
        let control =
            String::from("pub use std::{ sync::*, fmt::Debug, future::Future as AliasFuture };");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_module_import_definition() {
        let mod_import = Import::Module {
            public_visibility: true,
            name: String::from("my_module"),
        };
        let control = String::from("pub mod my_module;");
        assert_eq!(format!("{}", mod_import), control)
    }
}
