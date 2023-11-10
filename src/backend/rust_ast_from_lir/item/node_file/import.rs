use crate::lir::item::node_file::import::Import;
use crate::rust_ast::item::import::{Import as RustASTImport, PathTree};

/// Transform LIR import into RustAST import.
pub fn rust_ast_from_lir(import: Import) -> RustASTImport {
    match import {
        Import::NodeFile(module_name) => RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name,
                tree: Box::new(PathTree::Star),
            },
        },
        Import::Function(name) => RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("functions"),
                tree: Box::new(PathTree::Name { name, alias: None }),
            },
        },
        Import::Enumeration(name) => RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("typedefs"),
                tree: Box::new(PathTree::Name { name, alias: None }),
            },
        },
        Import::Structure(name) => RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("typedefs"),
                tree: Box::new(PathTree::Name { name, alias: None }),
            },
        },
        Import::ArrayAlias(name) => RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("typedefs"),
                tree: Box::new(PathTree::Name { name, alias: None }),
            },
        },
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::node_file::import::rust_ast_from_lir;
    use crate::lir::item::node_file::import::Import;
    use crate::rust_ast::item::import::{Import as RustASTImport, PathTree};

    #[test]
    fn should_create_rust_ast_import_from_lir_function_import() {
        let import = Import::Function(String::from("foo"));
        let control = RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("functions"),
                tree: Box::new(PathTree::Name {
                    name: String::from("foo"),
                    alias: None,
                }),
            },
        };
        assert_eq!(rust_ast_from_lir(import), control)
    }

    #[test]
    fn should_create_rust_ast_import_from_lir_node_import() {
        let import = Import::NodeFile(String::from("my_node"));
        let control = RustASTImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("my_node"),
                tree: Box::new(PathTree::Star),
            },
        };
        assert_eq!(rust_ast_from_lir(import), control)
    }
}
