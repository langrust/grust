use crate::{
    hir::typedef::{Typedef, TypedefKind},
    lir::item::{array_alias::ArrayAlias, enumeration::Enumeration, structure::Structure, Item},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Typedef {
    type LIR = Item;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            TypedefKind::Structure { fields, .. } => Item::Structure(Structure {
                name: symbol_table.get_name(self.id).clone(),
                fields: fields
                    .into_iter()
                    .map(|id| {
                        (
                            symbol_table.get_name(id).clone(),
                            symbol_table.get_type(id).clone(),
                        )
                    })
                    .collect(),
            }),
            TypedefKind::Enumeration { elements, .. } => Item::Enumeration(Enumeration {
                name: symbol_table.get_name(self.id).clone(),
                elements: elements
                    .into_iter()
                    .map(|id| symbol_table.get_name(id).clone())
                    .collect(),
            }),
            TypedefKind::Array => Item::ArrayAlias(ArrayAlias {
                name: symbol_table.get_name(self.id).clone(),
                array_type: symbol_table.get_array_type(self.id).clone(),
                size: symbol_table.get_array_size(self.id),
            }),
        }
    }
}
