prelude! {
    ast::{Colon, Typedef},
}

use super::HIRFromAST;

impl HIRFromAST for Typedef {
    type HIR = hir::Typedef;

    // precondition: typedefs are already stored in symbol table
    // postcondition: construct HIR typedef and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
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
                    .collect::<TRes<Vec<_>>>()?;

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Structure { fields: field_ids },
                    location,
                })
            }

            Typedef::Enumeration { ident, .. } => {
                let id = ident.to_string();
                let type_id = symbol_table.get_enum_id(&id, false, location.clone(), errors)?;
                let element_ids = symbol_table.get_enum_elements(type_id).clone();
                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Enumeration {
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

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Array,
                    location,
                })
            }
        }
    }
}
