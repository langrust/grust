use crate::{
    hir::pattern::{Pattern, PatternKind},
    lir::pattern::Pattern as LIRPattern,
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
}
