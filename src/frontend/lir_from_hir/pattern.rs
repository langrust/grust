use crate::{
    hir::pattern::{Pattern, PatternKind},
    lir::pattern::Pattern as LIRPattern,
    symbol_table::SymbolTable,
};

/// Transform HIR pattern into LIR item.
pub fn lir_from_hir(pattern: Pattern, symbol_table: &SymbolTable) -> LIRPattern {
    match pattern.kind {
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
                        lir_from_hir(pattern, symbol_table),
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
                .map(|element| lir_from_hir(element, symbol_table))
                .collect(),
        },
        PatternKind::Some { pattern } => LIRPattern::Some {
            pattern: Box::new(lir_from_hir(*pattern, symbol_table)),
        },
        PatternKind::None => LIRPattern::None,
        PatternKind::Default => LIRPattern::Default,
    }
}
