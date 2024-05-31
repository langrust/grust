use itertools::Itertools;

use crate::{
    common::r#type::Type,
    hir::pattern::{Pattern, PatternKind},
    lir::{item::import::Import, pattern::Pattern as LIRPattern},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Pattern {
    type LIR = LIRPattern;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            PatternKind::Identifier { id } => LIRPattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            PatternKind::Constant { constant } => LIRPattern::Literal { literal: constant },
            PatternKind::Typed { pattern, typing } => LIRPattern::Typed {
                pattern: Box::new(pattern.lir_from_hir(symbol_table)),
                typing,
            },
            PatternKind::Structure { id, fields } => LIRPattern::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, optional_pattern)| {
                        (
                            symbol_table.get_name(id).clone(),
                            optional_pattern.map_or(
                                LIRPattern::Identifier {
                                    name: symbol_table.get_name(id).clone(),
                                },
                                |pattern| pattern.lir_from_hir(symbol_table),
                            ),
                        )
                    })
                    .collect(),
            },
            PatternKind::Enumeration { enum_id, elem_id } => LIRPattern::Enumeration {
                enum_name: symbol_table.get_name(enum_id).clone(),
                elem_name: symbol_table.get_name(elem_id).clone(),
                element: None,
            },
            PatternKind::Tuple { elements } => LIRPattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.lir_from_hir(symbol_table))
                    .collect(),
            },
            PatternKind::Some { pattern } => LIRPattern::Some {
                pattern: Box::new(pattern.lir_from_hir(symbol_table)),
            },
            PatternKind::None => LIRPattern::None,
            PatternKind::Default => LIRPattern::Default,
            PatternKind::Event {
                event_enum_id,
                event_element_id,
                pattern,
            } => match symbol_table.get_type(event_element_id) {
                Type::SMEvent(_) => LIRPattern::enumeration(
                    symbol_table.get_name(event_enum_id).clone(),
                    symbol_table.get_name(event_element_id).clone(),
                    Some(pattern.lir_from_hir(symbol_table)),
                ),
                Type::SMTimeout(_) => LIRPattern::enumeration(
                    symbol_table.get_name(event_enum_id).clone(),
                    symbol_table.get_name(event_element_id).clone(),
                    Some(LIRPattern::ok(pattern.lir_from_hir(symbol_table))),
                ),
                _ => unreachable!(),
            },
            PatternKind::TimeoutEvent {
                event_enum_id,
                event_element_id,
            } => LIRPattern::enumeration(
                symbol_table.get_name(event_enum_id).clone(),
                symbol_table.get_name(event_element_id).clone(),
                Some(LIRPattern::err()),
            ),
            PatternKind::NoEvent { .. } => LIRPattern::Default,
        }
    }

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match &self.kind {
            PatternKind::Identifier { .. }
            | PatternKind::Constant { .. }
            | PatternKind::NoEvent { .. }
            | PatternKind::TimeoutEvent { .. }
            | PatternKind::None
            | PatternKind::Default => vec![],
            PatternKind::Structure { id, fields } => {
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
            PatternKind::Enumeration { enum_id, .. } => {
                vec![Import::Enumeration(symbol_table.get_name(*enum_id).clone())]
            }
            PatternKind::Tuple { elements } => elements
                .iter()
                .flat_map(|pattern| pattern.get_imports(symbol_table))
                .unique()
                .collect(),
            PatternKind::Some { pattern }
            | PatternKind::Typed { pattern, .. }
            | PatternKind::Event { pattern, .. } => pattern.get_imports(symbol_table),
        }
    }
}
