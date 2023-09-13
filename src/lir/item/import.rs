/// An import declaration.
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

/// A path of an `use` import.
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
