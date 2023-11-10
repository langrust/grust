use crate::lir::item::node_file::NodeFile;
use crate::rust_ast::file::File;
use crate::rust_ast::item::Item;

use self::import::rust_ast_from_lir as import_rust_ast_from_lir;
use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;

/// RustAST node and function import construction from LIR import.
pub mod import;
/// RustAST input structure construction from LIR input.
pub mod input;
/// RustAST state structure and implementation construction from LIR state.
pub mod state;

/// Transform LIR node_file into RustAST file.
pub fn rust_ast_from_lir(node_file: NodeFile) -> File {
    let mut items = node_file
        .imports
        .into_iter()
        .map(|import| Item::Import(import_rust_ast_from_lir(import)))
        .collect::<Vec<_>>();
    let input_structure = input_rust_ast_from_lir(node_file.input);
    let (state_structure, state_implementation) = state_rust_ast_from_lir(node_file.state);
    items.push(Item::Structure(input_structure));
    items.push(Item::Structure(state_structure));
    items.push(Item::Implementation(state_implementation));
    File {
        path: node_file.name,
        items,
    }
}
