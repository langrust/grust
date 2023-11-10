use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::rust_ast::item::structure::{Field, Structure as LIRStructure};
use crate::mir::item::structure::Structure;

/// Transform MIR structure into LIR structure.
pub fn lir_from_mir(structure: Structure) -> LIRStructure {
    let fields = structure
        .fields
        .into_iter()
        .map(|(name, r#type)| Field {
            public_visibility: true,
            name,
            r#type: type_lir_from_mir(r#type),
        })
        .collect();
    LIRStructure {
        public_visibility: true,
        name: structure.name,
        fields,
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::item::structure::lir_from_mir;
    use crate::rust_ast::item::structure::{Field, Structure as LIRStructure};
    use crate::rust_ast::r#type::Type as LIRType;
    use crate::mir::item::structure::Structure;

    #[test]
    fn should_create_lir_structure_from_mir_structure() {
        let structure = Structure {
            name: String::from("Point"),
            fields: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
        };
        let control = LIRStructure {
            public_visibility: true,
            name: String::from("Point"),
            fields: vec![
                Field {
                    public_visibility: true,
                    name: String::from("x"),
                    r#type: LIRType::Identifier {
                        identifier: String::from("i64"),
                    },
                },
                Field {
                    public_visibility: true,
                    name: String::from("y"),
                    r#type: LIRType::Identifier {
                        identifier: String::from("i64"),
                    },
                },
            ],
        };
        assert_eq!(lir_from_mir(structure), control)
    }
}
