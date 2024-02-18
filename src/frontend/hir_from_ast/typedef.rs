use crate::ast::typedef::{Typedef, TypedefKind};
use crate::error::{Error, TerminationError};
use crate::hir::typedef::{Typedef as HIRTypedef, TypedefKind as HIRTypedefKind};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Typedef {
    type HIR = HIRTypedef;

    // precondition: typedefs are already stored in symbol table
    // postcondition: construct HIR typedef and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Typedef { id, kind, location } = self;
        match kind {
            TypedefKind::Structure { fields } => {
                let type_id = symbol_table.get_struct_id(&id, false, location.clone(), errors)?;
                let field_ids = symbol_table.get_struct_fields(&type_id).clone();

                // insert field's type in symbol table
                field_ids
                    .iter()
                    .zip(fields)
                    .map(|(id, (name, typing))| {
                        assert!(name.eq(symbol_table.get_name(id)));
                        let typing = typing.hir_from_ast(&location, symbol_table, errors)?;
                        Ok(symbol_table.set_type(id, typing))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(HIRTypedef {
                    id: type_id,
                    kind: HIRTypedefKind::Structure { fields: field_ids },
                    location,
                })
            }

            TypedefKind::Enumeration { .. } => {
                let type_id = symbol_table.get_enum_id(&id, false, location.clone(), errors)?;
                let element_ids = symbol_table.get_enum_elements(&type_id).clone();
                Ok(HIRTypedef {
                    id: type_id,
                    kind: HIRTypedefKind::Enumeration {
                        elements: element_ids,
                    },
                    location,
                })
            }

            TypedefKind::Array { array_type, .. } => {
                let type_id = symbol_table.get_array_id(&id, false, location.clone(), errors)?;

                // insert array's type in symbol table
                let typing = array_type.hir_from_ast(&location, symbol_table, errors)?;
                symbol_table.set_array_type(&type_id, typing);

                Ok(HIRTypedef {
                    id: type_id,
                    kind: HIRTypedefKind::Array,
                    location,
                })
            }
        }
    }
}

impl Typedef {
    pub fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Typedef { id, kind, location } = self;

        match kind {
            TypedefKind::Structure { fields } => {
                symbol_table.local();

                let field_ids = fields
                    .iter()
                    .map(|(field_name, _)| {
                        let field_id = symbol_table.insert_identifier(
                            field_name.clone(),
                            None,
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(field_id)
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                symbol_table.global();

                let _ = symbol_table.insert_struct(
                    id.clone(),
                    field_ids.clone(),
                    false,
                    location.clone(),
                    errors,
                )?;
            }
            TypedefKind::Enumeration { elements } => {
                let element_ids = elements
                    .iter()
                    .map(|element_name| {
                        let element_id = symbol_table.insert_identifier(
                            format!("{id}::{element_name}"),
                            None,
                            false,
                            location.clone(),
                            errors,
                        )?;
                        Ok(element_id)
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                let _ = symbol_table.insert_enum(
                    id.clone(),
                    element_ids.clone(),
                    false,
                    location.clone(),
                    errors,
                )?;
            }
            TypedefKind::Array { size, .. } => {
                let _ = symbol_table.insert_array(
                    id.clone(),
                    None,
                    *size,
                    false,
                    location.clone(),
                    errors,
                )?;
            }
        }

        Ok(())
    }
}
