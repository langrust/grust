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

#[cfg(test)]
mod lir_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::item::node_file::input::lir_from_mir;
    use crate::lir::item::structure::{Structure, Field};
    use crate::lir::r#type::Type as LIRType;
    use crate::mir::item::node_file::input::{Input, InputElement};

    #[test]
    fn should_create_lir_structure_from_mir_node_input() {
        let input = Input {
            node_name: format!("Node"),
            elements: vec![
                InputElement { identifier: format!("i"), r#type: Type::Integer }
            ],
        };
        let control = Structure { public_visibility: true, name: format!("NodeInput"), fields: vec![
            Field { public_visibility: true, name: format!("i"), r#type: LIRType::Identifier {
                identifier: String::from("i64"),
            } }
        ] };
        assert_eq!(lir_from_mir(input), control)
    }
}
