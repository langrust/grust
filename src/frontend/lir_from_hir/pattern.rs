use itertools::Itertools;

use crate::{
    hir::pattern::{Pattern, PatternKind},
    lir::{item::node_file::import::Import, pattern::Pattern as LIRPattern},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Pattern {
    type LIR = LIRPattern;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            PatternKind::Identifier { id } => LIRPattern::Identifier {
                name: symbol_table.get_name(&id).clone(),
            },
            PatternKind::Constant { constant } => LIRPattern::Literal { literal: constant },
            PatternKind::Structure { id, fields } => LIRPattern::Structure {
                name: symbol_table.get_name(&id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, pattern)| {
                        (
                            symbol_table.get_name(&id).clone(),
                            pattern.lir_from_hir(symbol_table),
                        )
                    })
                    .collect(),
            },
            PatternKind::Enumeration { enum_id, elem_id } => LIRPattern::Enumeration {
                enum_name: symbol_table.get_name(&enum_id).clone(),
                elem_name: symbol_table.get_name(&elem_id).clone(),
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
        }
    }

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match &self.kind {
            PatternKind::Identifier { .. }
            | PatternKind::Constant { .. }
            | PatternKind::None
            | PatternKind::Default => vec![],
            PatternKind::Structure { id, fields } => {
                let mut imports = fields
                    .iter()
                    .flat_map(|(_, pattern)| pattern.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.push(Import::Structure(symbol_table.get_name(id).clone()));

                imports
            }
            PatternKind::Enumeration { enum_id, .. } => {
                vec![Import::Enumeration(symbol_table.get_name(enum_id).clone())]
            }
            PatternKind::Tuple { elements } => elements
                .iter()
                .flat_map(|pattern| pattern.get_imports(symbol_table))
                .unique()
                .collect(),
            PatternKind::Some { pattern } => pattern.get_imports(symbol_table),
        }
    }
}
