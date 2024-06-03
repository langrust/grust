prelude! {
    hir::typedef::{self, Typedef},
    lir::item::{ArrayAlias, Enumeration, Structure, Item},
}

use super::LIRFromHIR;

impl LIRFromHIR for Typedef {
    type LIR = Item;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            typedef::Kind::Structure { fields, .. } => Item::Structure(Structure {
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
            typedef::Kind::Enumeration { elements, .. } => Item::Enumeration(Enumeration {
                name: symbol_table.get_name(self.id).clone(),
                elements: elements
                    .into_iter()
                    .map(|id| symbol_table.get_name(id).clone())
                    .collect(),
            }),
            typedef::Kind::Array => Item::ArrayAlias(ArrayAlias {
                name: symbol_table.get_name(self.id).clone(),
                array_type: symbol_table.get_array_type(self.id).clone(),
                size: symbol_table.get_array_size(self.id),
            }),
        }
    }
}
