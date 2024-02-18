use std::collections::HashMap;

use crate::ast::pattern::{Pattern, PatternKind};
use crate::error::{Error, TerminationError};
use crate::hir::pattern::{Pattern as HIRPattern, PatternKind as HIRPatternKind};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Pattern {
    type HIR = HIRPattern;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Pattern { kind, location } = self;
        match kind {
            PatternKind::Constant { constant } => Ok(HIRPattern {
                kind: HIRPatternKind::Constant { constant },
                typing: None,
                location,
            }),
            PatternKind::Identifier { name } => {
                let id =
                    symbol_table.insert_identifier(name, None, true, location.clone(), errors)?;
                Ok(HIRPattern {
                    kind: HIRPatternKind::Identifier { id },
                    typing: None,
                    location,
                })
            }
            PatternKind::Structure { name, fields } => {
                let id = symbol_table.get_struct_id(&name, false, location.clone(), errors)?;
                let mut field_ids = symbol_table
                    .get_struct_fields(&id)
                    .clone()
                    .into_iter()
                    .map(|id| (symbol_table.get_name(&id).clone(), id))
                    .collect::<HashMap<_, _>>();

                let fields = fields
                    .into_iter()
                    .map(|(field_name, pattern)| {
                        let id = field_ids.remove(&field_name).map_or_else(
                            || {
                                let error = Error::UnknownField {
                                    structure_name: name.clone(),
                                    field_name: field_name.clone(),
                                    location: location.clone(),
                                };
                                errors.push(error);
                                Err(TerminationError)
                            },
                            |id| Ok(id),
                        )?;
                        let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                        Ok((id, pattern))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                // check if there are no missing fields
                field_ids
                    .keys()
                    .map(|field_name| {
                        let error = Error::MissingField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err::<(), TerminationError>(TerminationError)
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(HIRPattern {
                    kind: HIRPatternKind::Structure { id, fields },
                    typing: None,
                    location,
                })
            }
            PatternKind::Enumeration {
                enum_name,
                elem_name,
            } => {
                let enum_id =
                    symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
                let elem_id = symbol_table.get_identifier_id(
                    &format!("{enum_name}::{elem_name}"),
                    false,
                    location.clone(),
                    errors,
                )?;
                Ok(HIRPattern {
                    kind: HIRPatternKind::Enumeration { enum_id, elem_id },
                    typing: None,
                    location,
                })
            }
            PatternKind::Tuple { elements } => Ok(HIRPattern {
                kind: HIRPatternKind::Tuple {
                    elements: elements
                        .into_iter()
                        .map(|pattern| pattern.hir_from_ast(symbol_table, errors))
                        .collect::<Vec<Result<_, _>>>()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()?,
                },
                typing: None,
                location,
            }),
            PatternKind::Some { pattern } => Ok(HIRPattern {
                kind: HIRPatternKind::Some {
                    pattern: Box::new(pattern.hir_from_ast(symbol_table, errors)?),
                },
                typing: None,
                location,
            }),
            PatternKind::None => Ok(HIRPattern {
                kind: HIRPatternKind::None,
                typing: None,
                location,
            }),
            PatternKind::Default => Ok(HIRPattern {
                kind: HIRPatternKind::Default,
                typing: None,
                location,
            }),
        }
    }
}
