use crate::backend::rust_ast_from_lir::item::node_file::state::init::rust_ast_from_lir as init_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::item::node_file::state::step::rust_ast_from_lir as step_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::lir::item::node_file::state::{State, StateElement};
use crate::rust_ast::item::implementation::Implementation;
use crate::rust_ast::item::structure::{Field, Structure};
use crate::rust_ast::r#type::Type as RustASTType;

/// RustAST init method construction from LIR init.
pub mod init;
/// RustAST step method construction from LIR step.
pub mod step;

/// Transform LIR state into RustAST structure and implementation.
pub fn rust_ast_from_lir(state: State) -> (Structure, Implementation) {
    let fields = state
        .elements
        .into_iter()
        .map(|element| match element {
            StateElement::Buffer { identifier, r#type } => Field {
                public_visibility: false,
                name: identifier,
                r#type: type_rust_ast_from_lir(r#type),
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
        items: vec![
            init_rust_ast_from_lir(state.init),
            step_rust_ast_from_lir(state.step),
        ],
    };

    (structure, implementation)
}
