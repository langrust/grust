prelude! {
    ast::{Colon, Typedef},
}

impl IntoHir<hir::ctx::Simple<'_>> for Typedef {
    type Hir = hir::Typedef;

    // precondition: typedefs are already stored in symbol table
    // postcondition: construct HIR typedef and check identifiers good use
    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        let location = Location::default();
        match self {
            Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                let type_id = ctxt
                    .syms
                    .get_struct_id(&id, false, location.clone(), ctxt.errors)?;
                let field_ids = ctxt.syms.get_struct_fields(type_id).clone();

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
                            debug_assert_eq!(&name, ctxt.syms.get_name(*id));
                            let typing = typing.into_hir(&mut ctxt.add_loc(&location))?;
                            Ok(ctxt.syms.set_type(*id, typing))
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
                let type_id = ctxt
                    .syms
                    .get_enum_id(&id, false, location.clone(), ctxt.errors)?;
                let element_ids = ctxt.syms.get_enum_elements(type_id).clone();
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
                let type_id = ctxt
                    .syms
                    .get_array_id(&id, false, location.clone(), ctxt.errors)?;

                // insert array's type in symbol table
                let typing = array_type.into_hir(&mut ctxt.add_loc(&location))?;
                ctxt.syms.set_array_type(type_id, typing);

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Array,
                    location,
                })
            }
        }
    }
}
