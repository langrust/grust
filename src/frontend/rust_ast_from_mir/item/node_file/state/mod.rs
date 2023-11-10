use crate::frontend::rust_ast_from_mir::item::node_file::state::init::rust_ast_from_mir as init_rust_ast_from_mir;
use crate::frontend::rust_ast_from_mir::item::node_file::state::step::rust_ast_from_mir as step_rust_ast_from_mir;
use crate::frontend::rust_ast_from_mir::r#type::rust_ast_from_mir as type_rust_ast_from_mir;
use crate::rust_ast::item::implementation::Implementation;
use crate::rust_ast::item::structure::{Field, Structure};
use crate::rust_ast::r#type::Type as RustASTType;
use crate::lir::item::node_file::state::{State, StateElement};

/// RustAST init method construction from LIR init.
pub mod init;
/// RustAST step method construction from LIR step.
pub mod step;

/// Transform LIR state into RustAST structure and implementation.
pub fn rust_ast_from_mir(state: State) -> (Structure, Implementation) {
    let fields = state
        .elements
        .into_iter()
        .map(|element| match element {
            StateElement::Buffer { identifier, r#type } => Field {
                public_visibility: false,
                name: identifier,
                r#type: type_rust_ast_from_mir(r#type),
            },
            StateElement::CalledNode {
                identifier,
                node_name,
            } => Field {
                public_visibility: false,
                name: identifier,
                r#type: RustASTType::Identifier {
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
        items: vec![init_rust_ast_from_mir(state.init), step_rust_ast_from_mir(state.step)],
    };

    (structure, implementation)
}
