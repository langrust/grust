use crate::{
    hir::typedef::{Typedef, TypedefKind},
    lir::item::{array_alias::ArrayAlias, enumeration::Enumeration, structure::Structure, Item},
    symbol_table::SymbolTable,
};

/// Transform HIR typedef into LIR item.
pub fn lir_from_hir(typedef: Typedef, symbol_table: &SymbolTable) -> Item {
    match typedef.kind {
        TypedefKind::Structure { fields, .. } => Item::Structure(Structure {
            name: symbol_table.get_name(&typedef.id).clone(),
            fields: fields
                .into_iter()
                .map(|id| {
                    (
                        symbol_table.get_name(&typedef.id).clone(),
                        symbol_table.get_type(&typedef.id).clone(),
                    )
                })
                .collect(),
        }),
        TypedefKind::Enumeration { elements, .. } => Item::Enumeration(Enumeration {
            name: symbol_table.get_name(&typedef.id).clone(),
            elements: elements
                .into_iter()
                .map(|id| symbol_table.get_name(&id).clone())
                .collect(),
        }),
        TypedefKind::Array {
            array_type, size, ..
        } => Item::ArrayAlias(ArrayAlias {
            name: symbol_table.get_name(&typedef.id).clone(),
            array_type,
            size,
        }),
    }
}
