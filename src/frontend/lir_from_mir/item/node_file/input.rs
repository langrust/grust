use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::lir::item::structure::{Field, Structure};
use crate::mir::item::node_file::input::{Input, InputElement};

/// Transform MIR input into LIR structure.
pub fn lir_from_mir(input: Input) -> Structure {
    let fields = input
        .elements
        .into_iter()
        .map(|InputElement { identifier, r#type }| Field {
            public_visibility: true,
            name: identifier,
            r#type: type_lir_from_mir(r#type),
        })
        .collect();
    Structure {
        public_visibility: true,
        name: input.node_name + "Input",
        fields,
    }
}
