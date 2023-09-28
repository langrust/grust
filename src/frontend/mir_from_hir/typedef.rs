use crate::{
    ast::typedef::Typedef,
    mir::item::{array_alias::ArrayAlias, enumeration::Enumeration, structure::Structure, Item},
};

/// Transform HIR typedef into MIR item.
pub fn mir_from_hir(typedef: Typedef) -> Item {
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
