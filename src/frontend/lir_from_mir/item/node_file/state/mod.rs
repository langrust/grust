use crate::frontend::lir_from_mir::item::node_file::state::init::lir_from_mir as init_lir_from_mir;
use crate::frontend::lir_from_mir::item::node_file::state::step::lir_from_mir as step_lir_from_mir;
use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::rust_ast::item::implementation::Implementation;
use crate::rust_ast::item::structure::{Field, Structure};
use crate::rust_ast::r#type::Type as LIRType;
use crate::mir::item::node_file::state::{State, StateElement};

/// LIR init method construction from MIR init.
pub mod init;
/// LIR step method construction from MIR step.
pub mod step;

/// Transform MIR state into LIR structure and implementation.
pub fn lir_from_mir(state: State) -> (Structure, Implementation) {
    let fields = state
        .elements
        .into_iter()
        .map(|element| match element {
            StateElement::Buffer { identifier, r#type } => Field {
                public_visibility: false,
                name: identifier,
                r#type: type_lir_from_mir(r#type),
            },
            StateElement::CalledNode {
                identifier,
                node_name,
            } => Field {
                public_visibility: false,
                name: identifier,
                r#type: LIRType::Identifier {
                    identifier: node_name + "State",
                },
            },
        })
        .collect();
    let structure = Structure {
        public_visibility: false,
        name: state.node_name.clone() + "State",
        fields,
    };

    let implementation = Implementation {
        trait_name: None,
        type_name: state.node_name + "State",
        items: vec![init_lir_from_mir(state.init), step_lir_from_mir(state.step)],
    };

    (structure, implementation)
}
