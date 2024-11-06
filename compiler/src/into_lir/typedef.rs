prelude! {
    hir::typedef::{self, Typedef},
    lir::item::{ArrayAlias, Enumeration, Structure, Item},
}

impl IntoLir<&'_ SymbolTable> for Typedef {
    type Lir = Item;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
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
