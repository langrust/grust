prelude! { hir::{Pattern, pattern} }

use super::LIRFromHIR;

impl LIRFromHIR for Pattern {
    type LIR = lir::Pattern;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            pattern::Kind::Identifier { id } => lir::Pattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            pattern::Kind::Constant { constant } => lir::Pattern::Literal { literal: constant },
            pattern::Kind::Typed { pattern, typing } => lir::Pattern::Typed {
                pattern: Box::new(pattern.lir_from_hir(symbol_table)),
                typing,
            },
            pattern::Kind::Structure { id, fields } => lir::Pattern::Structure {
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
                                |pattern| pattern.lir_from_hir(symbol_table),
                            ),
                        )
                    })
                    .collect(),
            },
            pattern::Kind::Enumeration { enum_id, elem_id } => lir::Pattern::Enumeration {
                enum_name: symbol_table.get_name(enum_id).clone(),
                elem_name: symbol_table.get_name(elem_id).clone(),
                element: None,
            },
            pattern::Kind::Tuple { elements } => lir::Pattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.lir_from_hir(symbol_table))
                    .collect(),
            },
            pattern::Kind::Some { pattern } => lir::Pattern::Some {
                pattern: Box::new(pattern.lir_from_hir(symbol_table)),
            },
            pattern::Kind::None => lir::Pattern::None,
            pattern::Kind::Default => lir::Pattern::Default,
            pattern::Kind::PresentEvent { event_id, pattern } => {
                match symbol_table.get_type(event_id) {
                    Typ::SMEvent { .. } => lir::Pattern::some(pattern.lir_from_hir(symbol_table)),
                    Typ::SMTimeout { .. } => {
                        lir::Pattern::some(lir::Pattern::ok(pattern.lir_from_hir(symbol_table)))
                    }
                    _ => unreachable!(),
                }
            }
            pattern::Kind::TimeoutEvent { event_id } => match symbol_table.get_type(event_id) {
                Typ::SMTimeout { .. } => lir::Pattern::some(lir::Pattern::err()),
                _ => unreachable!(),
            },
            pattern::Kind::NoEvent { event_id } => match symbol_table.get_type(event_id) {
                Typ::SMEvent { .. } | Typ::SMTimeout { .. } => lir::Pattern::none(),
                _ => unreachable!(),
            },
        }
    }
}
