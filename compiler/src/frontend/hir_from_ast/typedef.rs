use crate::ast::colon::Colon;
use crate::ast::typedef::Typedef;
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::typedef::{Typedef as HIRTypedef, TypedefKind};
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
        let location = Location::default();
        match self {
            Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                let type_id = symbol_table.get_struct_id(&id, false, location.clone(), errors)?;
                let field_ids = symbol_table.get_struct_fields(type_id).clone();

                // insert field's type in symbol table
                field_ids
                    .iter()
                    .zip(fields)
                    .map(
                        |(
                            id,
                            Colon {
                                left: ident,
                                right: typing,
                                ..
                            },
                        )| {
                            let name = ident.to_string();
                            debug_assert_eq!(&name, symbol_table.get_name(*id));
                            let typing = typing.hir_from_ast(&location, symbol_table, errors)?;
                            Ok(symbol_table.set_type(*id, typing))
                        },
                    )
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(HIRTypedef {
                    id: type_id,
                    kind: TypedefKind::Structure { fields: field_ids },
                    location,
                })
            }

            Typedef::Enumeration { ident, .. } => {
                let id = ident.to_string();
                let type_id = symbol_table.get_enum_id(&id, false, location.clone(), errors)?;
                let element_ids = symbol_table.get_enum_elements(type_id).clone();
                Ok(HIRTypedef {
                    id: type_id,
                    kind: TypedefKind::Enumeration {
                        elements: element_ids,
                    },
                    location,
                })
            }

            Typedef::Array {
                ident, array_type, ..
            } => {
                let id = ident.to_string();
                let type_id = symbol_table.get_array_id(&id, false, location.clone(), errors)?;

                // insert array's type in symbol table
                let typing = array_type.hir_from_ast(&location, symbol_table, errors)?;
                symbol_table.set_array_type(type_id, typing);

                Ok(HIRTypedef {
                    id: type_id,
                    kind: TypedefKind::Array,
                    location,
                })
            }
        }
    }
}

impl Typedef {
    /// Store typedef's identifiers in symbol table.
    pub fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let location = Location::default();
        match self {
            Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                symbol_table.local();

                let field_ids = fields
                    .iter()
                    .map(|Colon { left: ident, .. }| {
                        let field_name = ident.to_string();
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
            Typedef::Enumeration {
                ident, elements, ..
            } => {
                let id = ident.to_string();
                let element_ids = elements
                    .iter()
                    .map(|element_ident| {
                        let element_name = element_ident.to_string();
                        let element_id = symbol_table.insert_enum_elem(
                            element_name.clone(),
                            id.clone(),
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
            Typedef::Array { ident, size, .. } => {
                let id = ident.to_string();
                let size = size.base10_parse().unwrap();
                let _ = symbol_table.insert_array(
                    id.clone(),
                    None,
                    size,
                    false,
                    location.clone(),
                    errors,
                )?;
            }
        }

        Ok(())
    }
}
