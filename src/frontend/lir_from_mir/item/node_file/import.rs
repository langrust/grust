use crate::lir::item::import::{Import as LIRImport, PathTree};
use crate::mir::item::node_file::import::Import;

/// Transform MIR import into LIR import.
pub fn lir_from_mir(import: Import) -> LIRImport {
    match import {
        Import::NodeFile(module_name) => LIRImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name,
                tree: Box::new(PathTree::Star),
            },
        },
        Import::Function(name) => LIRImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("functions"),
                tree: Box::new(PathTree::Name { name, alias: None }),
            },
        },
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::frontend::lir_from_mir::item::node_file::import::lir_from_mir;
    use crate::lir::item::import::{Import as LIRImport, PathTree};
    use crate::mir::item::node_file::import::Import;

    #[test]
    fn should_create_lir_import_from_mir_function_import() {
        let import = Import::Function(String::from("foo"));
        let control = LIRImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("functions"),
                tree: Box::new(PathTree::Name { name: String::from("foo"), alias: None }),
            }
        };
        assert_eq!(lir_from_mir(import), control)
    }

    #[test]
    fn should_create_lir_import_from_mir_node_import() {
        let import = Import::NodeFile(String::from("my_node"));
        let control = LIRImport::Use {
            public_visibility: false,
            tree: PathTree::Path {
                module_name: String::from("my_node"),
                tree: Box::new(PathTree::Star),
            },
        };
        assert_eq!(lir_from_mir(import), control)
    }
}
