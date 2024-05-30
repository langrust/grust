use self::event::rust_ast_from_lir as event_rust_ast_from_lir;
use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;
use crate::lir::item::state_machine::StateMachine;
use std::collections::BTreeSet;
use syn::*;

/// RustAST event structure construction from LIR event.
pub mod event;
/// RustAST input structure construction from LIR input.
pub mod input;
/// RustAST state structure and implementation construction from LIR state.
pub mod state;

/// Transform LIR state_machine into items.
pub fn rust_ast_from_lir(state_machine: StateMachine, crates: &mut BTreeSet<String>) -> Vec<Item> {
    let mut items = vec![];

    if let Some(event) = state_machine.event {
        let mut enum_items = event_rust_ast_from_lir(event);
        items.append(&mut enum_items);
    }

    let input_structure = input_rust_ast_from_lir(state_machine.input);
    items.push(Item::Struct(input_structure));

    let (state_structure, state_implementation) =
        state_rust_ast_from_lir(state_machine.state, crates);
    items.push(Item::Struct(state_structure));
    items.push(Item::Impl(state_implementation));

    items
}
