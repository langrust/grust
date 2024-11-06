prelude! {
    ast::Typ,
    syn::punctuated::{Pair, Punctuated},
}

impl<'a> HIRFromAST<hir::ctx::Loc<'a>> for Typ {
    type HIR = Typ;

    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<Typ> {
        // precondition: Typedefs are stored in symbol table
        // postcondition: construct a new Type without `Typ::NotDefinedYet`
        match self {
                Typ::Array { bracket_token, ty, semi_token, size } => Ok(Typ::Array {
                    bracket_token,
                    ty: Box::new(ty.hir_from_ast(ctxt)?),
                    semi_token,
                    size
                }),
                Typ::Tuple { paren_token, elements } => Ok(Typ::Tuple {
                    paren_token,
                    elements: elements.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.hir_from_ast(ctxt)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?
                }),
                Typ::NotDefinedYet(name) => ctxt.syms
                    .get_struct_id(&name.to_string(), false, ctxt.loc.clone(), &mut vec![])
                    .map(|id| Typ::Structure { name: name.clone(), id })
                    .or_else(|_| {
                        ctxt.syms
                            .get_enum_id(&name.to_string(), false, ctxt.loc.clone(), &mut vec![])
                            .map(|id| Typ::Enumeration { name: name.clone(), id })
                    })
                    .or_else(|_| {
                        let id = ctxt.syms.get_array_id(&name.to_string(), false, ctxt.loc.clone(), ctxt.errors)?;
                        Ok(ctxt.syms.get_array(id))
                    }),
                Typ::Abstract { paren_token, inputs, arrow_token, output } => {
                    let inputs = inputs.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.hir_from_ast(ctxt)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?;
                    let output = output.hir_from_ast(ctxt)?;
                    Ok(Typ::Abstract { paren_token, inputs, arrow_token, output: output.into() })
                }
                Typ::SMEvent { ty, question_token } => Ok(Typ::SMEvent {
                    ty: Box::new(ty.hir_from_ast(ctxt)?),
                    question_token
                }),
                Typ::Signal { signal_token, ty } => Ok(Typ::Signal {
                    signal_token,
                    ty: Box::new(ty.hir_from_ast(ctxt)?),
                }),
                Typ::Event { event_token, ty } => Ok(Typ::Event {
                    event_token,
                    ty: Box::new(ty.hir_from_ast(ctxt)?),
                }),
                Typ::Integer(_) | Typ::Float(_) | Typ::Boolean(_) | Typ::Unit(_) => Ok(self),
                Typ::Enumeration { .. }    // no enumeration at this time: they are `NotDefinedYet`
                | Typ::Structure { .. }    // no structure at this time: they are `NotDefinedYet`
                | Typ::Any                 // users can not write `Any` type
                | Typ::Polymorphism(_)     // users can not write `Polymorphism` type
                 => unreachable!(),
            }
    }
}
