use crate::rust_ast::file::File;
use crate::rust_ast::item::Item;
use crate::mir::item::node_file::NodeFile;

use self::import::lir_from_mir as import_lir_from_mir;
use self::input::lir_from_mir as input_lir_from_mir;
use self::state::lir_from_mir as state_lir_from_mir;

/// RustAST node and function import construction from MIR import.
pub mod import;
/// RustAST input structure construction from MIR input.
pub mod input;
/// RustAST state structure and implementation construction from MIR state.
pub mod state;

/// Transform MIR node_file into RustAST file.
pub fn lir_from_mir(node_file: NodeFile) -> File {
    let mut items = node_file
        .imports
        .into_iter()
        .map(|import| Item::Import(import_lir_from_mir(import)))
        .collect::<Vec<_>>();
    let input_structure = input_lir_from_mir(node_file.input);
    let (state_structure, state_implementation) = state_lir_from_mir(node_file.state);
    items.push(Item::Structure(input_structure));
    items.push(Item::Structure(state_structure));
    items.push(Item::Implementation(state_implementation));
    File {
        path: node_file.name,
        items,
    }
}
