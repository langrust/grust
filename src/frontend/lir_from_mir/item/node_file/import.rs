use crate::lir::item::import::{Import as LIRImport, PathTree};
use crate::mir::item::node_file::import::Import;

/// Transform MIR import into LIR import.
pub fn lir_from_mir(import: Import) -> LIRImport {
    match import {
        Import::NodeFile(name) => LIRImport::Use {
            public_visibility: false,
            tree: PathTree::Name { name, alias: None },
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
