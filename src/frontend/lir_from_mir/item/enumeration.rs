use crate::lir::item::enumeration::Enumeration as LIREnumeration;
use crate::mir::item::enumeration::Enumeration;

/// Transform MIR enumeration into LIR enumeration.
pub fn lir_from_mir(enumeration: Enumeration) -> LIREnumeration {
    LIREnumeration {
        public_visibility: true,
        name: enumeration.name,
        elements: enumeration.elements,
    }
}
