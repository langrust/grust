use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;
use crate::lir::item::node_file::NodeFile;
use std::collections::BTreeSet;
use syn::*;

/// RustAST input structure construction from LIR input.
pub mod input;
/// RustAST state structure and implementation construction from LIR state.
pub mod state;

/// Transform LIR node_file into items.
pub fn rust_ast_from_lir(node_file: NodeFile, crates: &mut BTreeSet<String>) -> Vec<Item> {
    let mut items = vec![];
    let input_structure = input_rust_ast_from_lir(node_file.input);
    let (state_structure, state_implementation) = state_rust_ast_from_lir(node_file.state, crates);
    items.push(Item::Struct(input_structure));
    items.push(Item::Struct(state_structure));
    items.push(Item::Impl(state_implementation));

    items
}
