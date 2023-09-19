use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::lir::item::structure::{Field, Structure as LIRStructure};
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
