prelude! { hir::{Pattern, pattern::Kind} }

impl IntoLir<&'_ SymbolTable> for Pattern {
    type Lir = lir::Pattern;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        match self.kind {
            Kind::Identifier { id } => lir::Pattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            Kind::Constant { constant } => lir::Pattern::Literal { literal: constant },
            Kind::Structure { id, fields } => lir::Pattern::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, optional_pattern)| {
                        (
                            symbol_table.get_name(id).clone(),
                            optional_pattern.map_or(
                                lir::Pattern::Identifier {
                                    name: symbol_table.get_name(id).clone(),
                                },
                                |pattern| pattern.into_lir(symbol_table),
                            ),
                        )
                    })
                    .collect(),
            },
            Kind::Enumeration { enum_id, elem_id } => lir::Pattern::Enumeration {
                enum_name: symbol_table.get_name(enum_id).clone(),
                elem_name: symbol_table.get_name(elem_id).clone(),
                element: None,
            },
            Kind::Tuple { elements } => lir::Pattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
            Kind::Some { pattern } => lir::Pattern::Some {
                pattern: Box::new(pattern.into_lir(symbol_table)),
            },
            Kind::None => lir::Pattern::None,
            Kind::Default => lir::Pattern::Default,
            Kind::PresentEvent { event_id, pattern } => match symbol_table.get_typ(event_id) {
                Typ::SMEvent { .. } => lir::Pattern::some(pattern.into_lir(symbol_table)),
                _ => unreachable!(),
            },
            Kind::NoEvent { event_id } => match symbol_table.get_typ(event_id) {
                Typ::SMEvent { .. } => lir::Pattern::none(),
                _ => unreachable!(),
            },
        }
    }
}

impl IntoLir<&'_ SymbolTable> for hir::stmt::Pattern {
    type Lir = lir::Pattern;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        match self.kind {
            hir::stmt::Kind::Identifier { id } => lir::Pattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            hir::stmt::Kind::Typed { id, typ } => lir::Pattern::Typed {
                pattern: Box::new(lir::Pattern::Identifier {
                    name: symbol_table.get_name(id).clone(),
                }),
                typ,
            },
            hir::stmt::Kind::Tuple { elements } => lir::Pattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
        }
    }
}
