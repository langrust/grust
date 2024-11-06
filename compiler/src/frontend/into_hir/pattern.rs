mod expr_pattern {
    prelude! {
        ast::expr::{PatEnumeration, PatStructure, PatTuple},
    }

    impl IntoHir<hir::ctx::Loc<'_>> for PatStructure {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let PatStructure { name, fields, rest } = self;

            let id = ctxt
                .syms
                .get_struct_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;
            let mut field_ids = ctxt
                .syms
                .get_struct_fields(id)
                .clone()
                .into_iter()
                .map(|id| (ctxt.syms.get_name(id).clone(), id))
                .collect::<HashMap<_, _>>();

            let fields = fields
                .into_iter()
                .map(|(field_name, optional_pattern)| {
                    let id = field_ids.remove(&field_name).map_or_else(
                        || {
                            let error = Error::UnknownField {
                                structure_name: name.clone(),
                                field_name: field_name.clone(),
                                location: ctxt.loc.clone(),
                            };
                            ctxt.errors.push(error);
                            Err(TerminationError)
                        },
                        |id| Ok(id),
                    )?;
                    let pattern = optional_pattern
                        .map(|pattern| pattern.into_hir(ctxt))
                        .transpose()?;
                    Ok((id, pattern))
                })
                .collect::<TRes<Vec<_>>>()?;

            if rest.is_none() {
                // check if there are no missing fields
                field_ids
                    .keys()
                    .map(|field_name| {
                        let error = Error::MissingField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: ctxt.loc.clone(),
                        };
                        ctxt.errors.push(error);
                        TRes::<()>::Err(TerminationError)
                    })
                    .collect::<TRes<Vec<_>>>()?;
            }

            Ok(hir::pattern::Kind::Structure { id, fields })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for PatEnumeration {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let PatEnumeration {
                enum_name,
                elem_name,
            } = self;

            let enum_id =
                ctxt.syms
                    .get_enum_id(&enum_name, false, ctxt.loc.clone(), ctxt.errors)?;
            let elem_id = ctxt.syms.get_enum_elem_id(
                &elem_name,
                &enum_name,
                false,
                ctxt.loc.clone(),
                ctxt.errors,
            )?;
            Ok(hir::pattern::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for PatTuple {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let PatTuple { elements } = self;
            Ok(hir::pattern::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|pattern| pattern.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for ast::expr::Pattern {
        type Hir = hir::Pattern;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<Self::Hir> {
            let kind = match self {
                ast::expr::Pattern::Constant(constant) => hir::pattern::Kind::Constant { constant },
                ast::expr::Pattern::Identifier(name) => {
                    let id =
                        ctxt.syms
                            .get_identifier_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;
                    hir::pattern::Kind::Identifier { id }
                }
                ast::expr::Pattern::Structure(pattern) => pattern.into_hir(ctxt)?,
                ast::expr::Pattern::Enumeration(pattern) => pattern.into_hir(ctxt)?,
                ast::expr::Pattern::Tuple(pattern) => pattern.into_hir(ctxt)?,
                // Pattern::None => hir::pattern::Kind::None,
                ast::expr::Pattern::Default => hir::pattern::Kind::Default,
            };

            Ok(hir::Pattern {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
            })
        }
    }
}

mod stmt_pattern {
    prelude! {
        ast::stmt::{Typed, Tuple},
    }

    impl IntoHir<hir::ctx::Loc<'_>> for Typed {
        type Hir = hir::stmt::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::stmt::Kind> {
            let Typed { ident, typing, .. } = self;

            let id = ctxt.syms.get_identifier_id(
                &ident.to_string(),
                false,
                ctxt.loc.clone(),
                ctxt.errors,
            )?;
            let typing = typing.into_hir(ctxt)?;
            Ok(hir::stmt::Kind::Typed { id, typing })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for Tuple {
        type Hir = hir::stmt::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::stmt::Kind> {
            let Tuple { elements } = self;
            Ok(hir::stmt::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|pattern| pattern.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for ast::stmt::Pattern {
        type Hir = hir::stmt::Pattern;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<Self::Hir> {
            let kind = match self {
                ast::stmt::Pattern::Identifier(ident) => {
                    let id = ctxt.syms.get_identifier_id(
                        &ident.to_string(),
                        false,
                        ctxt.loc.clone(),
                        ctxt.errors,
                    )?;
                    hir::stmt::Kind::Identifier { id }
                }
                ast::stmt::Pattern::Typed(pattern) => pattern.into_hir(ctxt)?,
                ast::stmt::Pattern::Tuple(pattern) => pattern.into_hir(ctxt)?,
            };

            Ok(hir::stmt::Pattern {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
            })
        }
    }
}
