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

#[cfg(test)]
mod lir_from_mir {
    use crate::frontend::lir_from_mir::item::enumeration::lir_from_mir;
    use crate::lir::item::enumeration::Enumeration as LIREnumeration;
    use crate::mir::item::enumeration::Enumeration;

    #[test]
    fn should_create_lir_enumeration_from_mir_enumeration() {
        let enumeration = Enumeration {
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Green"),
            ],
        };
        let control = LIREnumeration {
            public_visibility: true,
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Green"),
            ],
        };
        assert_eq!(lir_from_mir(enumeration), control)
    }
}
