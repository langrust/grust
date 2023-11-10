use crate::{
    ast::typedef::Typedef,
    lir::item::{array_alias::ArrayAlias, enumeration::Enumeration, structure::Structure, Item},
};

/// Transform HIR typedef into LIR item.
pub fn lir_from_hir(typedef: Typedef) -> Item {
    match typedef {
        Typedef::Structure { id, fields, .. } => Item::Structure(Structure { name: id, fields }),
        Typedef::Enumeration { id, elements, .. } => {
            Item::Enumeration(Enumeration { name: id, elements })
        }
        Typedef::Array {
            id,
            array_type,
            size,
            ..
        } => Item::ArrayAlias(ArrayAlias {
            name: id,
            array_type,
            size,
        }),
    }
}

#[cfg(test)]
mod lir_from_hir {
    use crate::{
        ast::typedef::Typedef,
        common::{location::Location, r#type::Type},
        frontend::lir_from_hir::typedef::lir_from_hir,
        lir::item::{
            array_alias::ArrayAlias, enumeration::Enumeration, structure::Structure, Item,
        },
    };

    #[test]
    fn should_transform_hir_structure_type_definition_into_mir_structure_item() {
        let structure = Typedef::Structure {
            id: format!("Point"),
            fields: vec![(format!("x"), Type::Integer), (format!("y"), Type::Integer)],
            location: Location::default(),
        };
        let control = Item::Structure(Structure {
            name: format!("Point"),
            fields: vec![(format!("x"), Type::Integer), (format!("y"), Type::Integer)],
        });
        assert_eq!(lir_from_hir(structure), control)
    }

    #[test]
    fn should_transform_hir_enumeration_type_definition_into_mir_enumeration_item() {
        let enumeration = Typedef::Enumeration {
            id: format!("Color"),
            elements: vec![format!("Red"), format!("Bleu"), format!("Green")],
            location: Location::default(),
        };
        let control = Item::Enumeration(Enumeration {
            name: format!("Color"),
            elements: vec![format!("Red"), format!("Bleu"), format!("Green")],
        });
        assert_eq!(lir_from_hir(enumeration), control)
    }

    #[test]
    fn should_transform_hir_array_type_definition_into_mir_array_alias_item() {
        let array = Typedef::Array {
            id: format!("M5x5"),
            array_type: Type::Array(Box::new(Type::Integer), 5),
            size: 5,
            location: Location::default(),
        };
        let control = Item::ArrayAlias(ArrayAlias {
            name: format!("M5x5"),
            array_type: Type::Array(Box::new(Type::Integer), 5),
            size: 5,
        });
        assert_eq!(lir_from_hir(array), control)
    }
}
