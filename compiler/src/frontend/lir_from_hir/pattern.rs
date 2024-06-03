use itertools::Itertools;

prelude! {
    hir::{Pattern, pattern}, lir::item::Import,
}

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
            pattern::Kind::Event {
                event_enum_id,
                event_element_id,
                pattern,
            } => match symbol_table.get_type(event_element_id) {
                Typ::SMEvent(_) => lir::Pattern::enumeration(
                    symbol_table.get_name(event_enum_id).clone(),
                    symbol_table.get_name(event_element_id).clone(),
                    Some(pattern.lir_from_hir(symbol_table)),
                ),
                Typ::SMTimeout(_) => lir::Pattern::enumeration(
                    symbol_table.get_name(event_enum_id).clone(),
                    symbol_table.get_name(event_element_id).clone(),
                    Some(lir::Pattern::ok(pattern.lir_from_hir(symbol_table))),
                ),
                _ => unreachable!(),
            },
            pattern::Kind::TimeoutEvent {
                event_enum_id,
                event_element_id,
            } => lir::Pattern::enumeration(
                symbol_table.get_name(event_enum_id).clone(),
                symbol_table.get_name(event_element_id).clone(),
                Some(lir::Pattern::err()),
            ),
            pattern::Kind::NoEvent { .. } => lir::Pattern::Default,
        }
    }

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match &self.kind {
            pattern::Kind::Identifier { .. }
            | pattern::Kind::Constant { .. }
            | pattern::Kind::NoEvent { .. }
            | pattern::Kind::TimeoutEvent { .. }
            | pattern::Kind::None
            | pattern::Kind::Default => vec![],
            pattern::Kind::Structure { id, fields } => {
                let mut imports = fields
                    .iter()
                    .flat_map(|(_, optional_pattern)| {
                        optional_pattern
                            .as_ref()
                            .map_or(vec![], |pattern| pattern.get_imports(symbol_table))
                    })
                    .unique()
                    .collect::<Vec<_>>();
                imports.push(Import::Structure(symbol_table.get_name(*id).clone()));

                imports
            }
            pattern::Kind::Enumeration { enum_id, .. } => {
                vec![Import::Enumeration(symbol_table.get_name(*enum_id).clone())]
            }
            pattern::Kind::Tuple { elements } => elements
                .iter()
                .flat_map(|pattern| pattern.get_imports(symbol_table))
                .unique()
                .collect(),
            pattern::Kind::Some { pattern }
            | pattern::Kind::Typed { pattern, .. }
            | pattern::Kind::Event { pattern, .. } => pattern.get_imports(symbol_table),
        }
    }
}
