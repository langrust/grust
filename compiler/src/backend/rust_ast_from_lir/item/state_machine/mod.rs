use std::collections::BTreeSet;

prelude! { just
    lir::item::state_machine::StateMachine, syn,
}

use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;

/// RustAST input structure construction from LIR input.
pub mod input;
/// RustAST state structure and implementation construction from LIR state.
pub mod state;

/// Transform LIR state_machine into items.
pub fn rust_ast_from_lir(
    state_machine: StateMachine,
    crates: &mut BTreeSet<String>,
) -> Vec<syn::Item> {
    let mut items = vec![];

    let input_structure = input_rust_ast_from_lir(state_machine.input);
    items.push(syn::Item::Struct(input_structure));

    let (state_structure, state_implementation) =
        state_rust_ast_from_lir(state_machine.state, crates);
    items.push(syn::Item::Struct(state_structure));
    items.push(syn::Item::Impl(state_implementation));

    items
}
