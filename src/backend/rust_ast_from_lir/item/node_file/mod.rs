use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::item::import::rust_ast_from_lir as import_rust_ast_from_lir;
use crate::lir::item::node_file::NodeFile;
use syn::*;

/// RustAST input structure construction from LIR input.
pub mod input;
/// RustAST state structure and implementation construction from LIR state.
pub mod state;

/// Transform LIR node_file into RustAST file.
pub fn rust_ast_from_lir(node_file: NodeFile) -> (String, File) {
    let mut items = node_file
        .imports
        .into_iter()
        .map(|import| Item::Use(import_rust_ast_from_lir(import)))
        .collect::<Vec<_>>();
    let input_structure = input_rust_ast_from_lir(node_file.input);
    let (state_structure, state_implementation) = state_rust_ast_from_lir(node_file.state);
    items.push(Item::Struct(input_structure));
    items.push(Item::Struct(state_structure));
    items.push(Item::Impl(state_implementation));
    (
        format!("src/{}.rs", node_file.name),
        File {
            // path: ,
            items,
            shebang: None,
            attrs: Default::default(),
        },
    )
}
