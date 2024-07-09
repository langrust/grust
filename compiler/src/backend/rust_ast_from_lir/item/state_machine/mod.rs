use std::collections::BTreeSet;

prelude! { just
    syn::*,
    lir::item::state_machine::StateMachine,
}

use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;

/// RustAST input structure construction from LIR input.
pub mod input;
/// RustAST state structure and implementation construction from LIR state.
pub mod state;

/// Transform LIR state_machine into items.
pub fn rust_ast_from_lir(state_machine: StateMachine, crates: &mut BTreeSet<String>) -> Vec<Item> {
    let mut items = vec![];

    let input_structure = input_rust_ast_from_lir(state_machine.input);
    items.push(Item::Struct(input_structure));

    let (state_structure, state_implementation) =
        state_rust_ast_from_lir(state_machine.state, crates);
    items.push(Item::Struct(state_structure));
    items.push(Item::Impl(state_implementation));

    items
}
