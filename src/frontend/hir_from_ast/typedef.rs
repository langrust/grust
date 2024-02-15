use crate::ast::typedef::{Typedef, TypedefKind};
use crate::error::{Error, TerminationError};
use crate::hir::typedef::{Typedef as HIRTypedef, TypedefKind as HIRTypedefKind};
use crate::symbol_table::SymbolTable;

/// Transform AST typedef into HIR typedef.
pub fn hir_from_ast(
    typedef: Typedef,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRTypedef, TerminationError> {
    let Typedef { id, kind, location } = typedef;
    match kind {
        TypedefKind::Structure { fields } => {
            let field_ids = fields
                .into_iter()
                .map(|(field_name, field_type)| {
                    symbol_table.local();
                    let field_id = symbol_table.insert_identifier(
                        field_name,
                        Some(field_type),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    symbol_table.global();
                    Ok(field_id)
                })
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            let type_id = symbol_table.insert_struct(
                id.clone(),
                field_ids.clone(),
                false,
                location.clone(),
                errors,
            )?;
            Ok(HIRTypedef {
                id: type_id,
                kind: HIRTypedefKind::Structure { fields: field_ids },
                location,
            })
        }

        TypedefKind::Enumeration { elements } => {
            let element_ids = elements
                .into_iter()
                .map(|element_name| {
                    symbol_table.local();
                    let element_id = symbol_table.insert_identifier(
                        element_name,
                        None,
                        true,
                        location.clone(),
                        errors,
                    )?;
                    symbol_table.global();
                    Ok(element_id)
                })
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            let type_id = symbol_table.insert_enum(
                id.clone(),
                element_ids.clone(),
                false,
                location.clone(),
                errors,
            )?;
            // TODO: should not we store elements too?
            Ok(HIRTypedef {
                id: type_id,
                kind: HIRTypedefKind::Enumeration {
                    elements: element_ids,
                },
                location,
            })
        }

        TypedefKind::Array { array_type, size } => {
            let type_id = symbol_table.insert_array(
                id.clone(),
                array_type.clone(),
                size,
                false,
                location.clone(),
                errors,
            )?;
            Ok(HIRTypedef {
                id: type_id,
                kind: HIRTypedefKind::Array { array_type, size },
                location,
            })
        }
    }
}
