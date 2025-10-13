prelude! { ir0::{Component, Colon} }

pub trait Ir0Store {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()>;
}

impl Ir0Store for Component {
    /// Store component's idents in symbol table.
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ctx.add_loc(loc);

        ctx.local();

        // store input idents and get their ids
        let inputs = self
            .args
            .iter()
            .map(
                |Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id = ctx.ctx0.insert_ident(
                        ident.clone(),
                        Scope::Input,
                        Some(typ),
                        true,
                        ctx.errors,
                    )?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        // store outputs and get their ids
        let outputs = self
            .outs
            .iter()
            .map(
                |Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id = ctx.ctx0.insert_ident(
                        ident.clone(),
                        Scope::Output,
                        Some(typ),
                        true,
                        ctx.errors,
                    )?;
                    Ok((self.ident.clone(), id)) // TODO: ident.clone()
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        // store locals and get their ids
        let locals = {
            let mut locals = HashMap::with_capacity(25);
            for equation in self.equations.iter() {
                equation.store_idents(false, &mut locals, &mut ctx.rm_loc())?;
            }
            locals.shrink_to_fit();
            locals
        };

        // store inits and get their ids
        let inits = {
            let mut inits = HashMap::with_capacity(5);
            for equation in self.equations.iter() {
                equation.store_inits(&mut inits, &mut ctx.rm_loc())?;
            }
            inits.shrink_to_fit();
            inits
        };

        ctx.global();

        let _ = ctx.ctx0.insert_comp(
            self.ident.clone(),
            (inputs, outputs),
            Some((locals, inits)),
            None,
            self.weight,
            ctx.errors,
        )?;

        Ok(())
    }
}

pub trait Ir0StoreIdents {
    /// Creates identifiers for the equation (depending on the config `store_outputs`)
    ///
    /// # Example
    ///
    /// ```grust
    /// match e {
    ///     pat1 => { let a: int = e_a; let b: float = e_b; },
    ///     pat2 => { let (a: int, b: float) = e_ab; },
    /// }
    /// ```
    ///
    /// With the above equations, the algorithm insert in the `idents` map
    /// [ a -> id_a ] and [ b -> id_b ].
    fn store_idents(
        &self,
        store_outputs: bool,
        idents: &mut HashMap<Ident, usize>,
        ctx: &mut ctx::Simple,
    ) -> TRes<()>;

    fn store_inits(&self, inits: &mut HashMap<Ident, usize>, ctx: &mut ctx::Simple) -> TRes<()>;

    /// Collects the identifiers of the equation.
    fn get_idents(
        &self,
        idents: &mut HashMap<Ident, ir0::stmt::Pattern>,
        ctx: &mut ctx::Simple,
    ) -> TRes<()>;

    /// Collects the initializations.
    fn get_inits(
        &self,
        inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
        ctx: &mut ctx::Simple,
    ) -> TRes<()>;
}

pub trait Ir0StoreInit: Sized {
    fn store_inits(&self, inits: &mut HashMap<Ident, usize>, ctx: &mut ctx::Simple) -> TRes<()>;

    fn get_inits(
        &self,
        inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
        ctx: &mut ctx::Simple,
    ) -> TRes<()>;
}

mod equation {
    prelude! { ir0::equation::* }

    impl Ir0StoreIdents for Eq {
        fn store_idents(
            &self,
            store_outputs: bool,
            idents: &mut HashMap<Ident, usize>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                // when output definitions should be stored
                Eq::OutputDef(instantiation) if store_outputs => instantiation
                    .pattern
                    .store(false, ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                // when output definitions are already stored (as component outputs)
                Eq::OutputDef(_) => Ok(()),
                Eq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .store(true, ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                Eq::MatchEq(MatchEq { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations.iter() {
                        eq.store_idents(store_outputs, idents, ctx)?;
                    }
                    Ok(())
                }
            }
        }

        fn store_inits(
            &self,
            _inits: &mut HashMap<Ident, usize>,
            _ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            Ok(())
        }

        fn get_idents(
            &self,
            idents: &mut HashMap<Ident, ir0::stmt::Pattern>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                Eq::OutputDef(instantiation) => instantiation
                    .pattern
                    .get_idents(ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                Eq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .get_idents(ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                Eq::MatchEq(MatchEq { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations {
                        eq.get_idents(idents, ctx)?;
                    }
                    Ok(())
                }
            }
        }

        fn get_inits(
            &self,
            _inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
            _ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            Ok(())
        }
    }

    impl Ir0StoreInit for InitArmWhen {
        fn store_inits(
            &self,
            inits: &mut HashMap<Ident, usize>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            for init in &self.equations {
                let idents = init.pattern.store_inits(ctx)?;
                inits.extend(idents);
            }
            Ok(())
        }

        fn get_inits(
            &self,
            inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            for init in &self.equations {
                let idents = init.pattern.get_idents(ctx)?;
                inits.extend(idents);
            }
            Ok(())
        }
    }

    impl Ir0StoreIdents for ReactEq {
        fn store_idents(
            &self,
            store_outputs: bool,
            idents: &mut HashMap<Ident, usize>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                // output definitions should be stored
                ReactEq::OutputDef(instantiation) if store_outputs => instantiation
                    .pattern
                    .store(false, ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                // when output definitions are already stored (as component's outputs)
                ReactEq::OutputDef(_) | ReactEq::Init(_) | ReactEq::Log(_) => Ok(()),
                ReactEq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .store(true, ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                ReactEq::MatchEq(MatchEq { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations.iter() {
                        eq.store_idents(store_outputs, idents, ctx)?;
                    }
                    Ok(())
                }
                ReactEq::WhenEq(WhenEq { arms, .. }) => {
                    // we want to collect every identifier, but events might be declared in only one
                    // branch then, it is needed to explore all branches
                    let mut when_idents = HashMap::new();
                    let mut add_idents = |equations: &[Eq]| {
                        // some flows are defined in multiple branches so we don't want them to trigger
                        // the *duplicated definition* error.
                        ctx.local();
                        for eq in equations {
                            eq.store_idents(store_outputs, &mut when_idents, ctx)?;
                        }
                        ctx.global();
                        Ok(())
                    };
                    for EventArmWhen { equations, .. } in arms {
                        add_idents(equations)?
                    }
                    // put the identifiers back in context
                    for (k, v) in when_idents.into_iter() {
                        let loc = k.loc();
                        if let std::collections::hash_map::Entry::Vacant(e) = idents.entry(k) {
                            e.insert(v);
                            ctx.ctx0.put_back_in_context(v, false, loc, ctx.errors)?;
                        }
                    }
                    Ok(())
                }
            }
        }

        fn store_inits(
            &self,
            inits: &mut HashMap<Ident, usize>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                ReactEq::Init(init) => init
                    .pattern
                    .store_inits(ctx)
                    .map(|idents| inits.extend(idents)),
                ReactEq::OutputDef(instantiation) => {
                    if let ir0::stream::ReactExpr::WhenExpr(ir0::stream::WhenExpr {
                        init: Some(_),
                        ..
                    }) = instantiation.expr
                    {
                        instantiation
                            .pattern
                            .store_inits(ctx)
                            .map(|idents| inits.extend(idents))?;
                    }
                    Ok(())
                }
                ReactEq::LocalDef(declaration) => {
                    if let ir0::stream::ReactExpr::WhenExpr(ir0::stream::WhenExpr {
                        init: Some(_),
                        ..
                    }) = declaration.expr
                    {
                        declaration
                            .typed_pattern
                            .store_inits(ctx)
                            .map(|idents| inits.extend(idents))?;
                    }
                    Ok(())
                }
                ReactEq::MatchEq(_) | ReactEq::Log(_) => Ok(()),
                ReactEq::WhenEq(WhenEq { init, .. }) => {
                    // store initializations
                    if let Some(init) = init {
                        init.store_inits(inits, ctx)?;
                    }
                    Ok(())
                }
            }
        }

        fn get_idents(
            &self,
            idents: &mut HashMap<Ident, ir0::stmt::Pattern>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                ReactEq::OutputDef(instantiation) => instantiation
                    .pattern
                    .get_idents(ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                ReactEq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .get_idents(ctx)
                    .map(|new_idents| idents.extend(new_idents)),
                ReactEq::MatchEq(MatchEq { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations {
                        eq.get_idents(idents, ctx)?;
                    }
                    Ok(())
                }
                ReactEq::WhenEq(WhenEq { arms, .. }) => {
                    let mut add_idents = |equations: &[Eq]| {
                        // we want to collect every identifier, but events might be declared in only
                        // one branch then, it is needed to explore all branches
                        for eq in equations {
                            eq.get_idents(idents, ctx)?;
                        }
                        Ok(())
                    };
                    for EventArmWhen { equations, .. } in arms {
                        add_idents(equations)?
                    }
                    Ok(())
                }
                ReactEq::Init(_) | ReactEq::Log(_) => Ok(()),
            }
        }

        fn get_inits(
            &self,
            inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                ReactEq::OutputDef(_)
                | ReactEq::LocalDef(_)
                | ReactEq::MatchEq(_)
                | ReactEq::Log(_) => Ok(()),
                ReactEq::WhenEq(WhenEq { init, .. }) => {
                    if let Some(init) = init {
                        init.get_inits(inits, ctx)
                    } else {
                        Ok(())
                    }
                }
                ReactEq::Init(init) => init
                    .pattern
                    .get_idents(ctx)
                    .map(|idents| inits.extend(idents)),
            }
        }
    }
}

impl Ir0Store for Ast {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        // store type definition first, so they can be used after
        self.items
            .iter()
            .filter_map(|item| match item {
                ir0::Item::Typedef(typedef) => Some(typedef),
                _ => None,
            })
            .map(|typedef| typedef.store(ctx))
            .collect::<TRes<Vec<_>>>()?;

        self.items
            .iter()
            .map(|item| match item {
                ir0::Item::Component(component) => component.store(ctx),
                ir0::Item::Function(function) => function.store(ctx),
                ir0::Item::ExtFun(extfun) => extfun.store(ctx),
                ir0::Item::ExtComp(extcomp) => extcomp.store(ctx),
                ir0::Item::Const(const_decl) => const_decl.store(ctx),
                ir0::Item::Typedef(_) // already stored
                | ir0::Item::Service(_)
                | ir0::Item::Import(_)
                | ir0::Item::Export(_) => Ok(()),
            })
            .collect::<TRes<Vec<_>>>()?;
        Ok(())
    }
}

impl Ir0Store for ir0::Function {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ctx.add_loc(loc);
        ctx.local();

        let inputs = self
            .args
            .iter()
            .map(
                |ir0::Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id =
                        ctx.ctx0
                            .insert_local_ident(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        ctx.global();

        let _ =
            ctx.ctx0
                .insert_function(self.ident.clone(), inputs, None, self.weight, ctx.errors)?;

        Ok(())
    }
}

impl Ir0Store for ir0::ExtFunDecl {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ctx.add_loc(loc);
        ctx.local();

        let inputs = self
            .args
            .iter()
            .map(
                |ir0::Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id =
                        ctx.ctx0
                            .insert_local_ident(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        ctx.global();

        let _ = ctx.ctx0.insert_function(
            self.ident.clone(),
            inputs,
            Some(self.path.clone()),
            self.weight,
            ctx.errors,
        )?;

        Ok(())
    }
}

impl Ir0Store for ir0::ExtCompDecl {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ctx.add_loc(loc);
        ctx.local();

        let inputs = self
            .args
            .iter()
            .map(
                |ir0::Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id =
                        ctx.ctx0
                            .insert_local_ident(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        let outputs = self
            .outs
            .iter()
            .map(
                |ir0::Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id =
                        ctx.ctx0
                            .insert_local_ident(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok((self.ident.clone(), id))
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        ctx.global();

        let _ = ctx.ctx0.insert_comp(
            self.ident.clone(),
            (inputs, outputs),
            None,
            Some(self.path.clone()),
            self.weight,
            ctx.errors,
        )?;
        // let _ = ctx.ctx0.insert_comp(
        //     self.ident.clone(),
        //     inputs,
        //     outputs,
        //     Some(locals),
        //     Some(inits),
        //     None,
        //     self.weight,
        //     ctx.errors,
        // )?;

        Ok(())
    }
}

impl Ir0Store for ir0::ConstDecl {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ctx.add_loc(loc);

        let _id = ctx.ctx0.insert_constant(
            self.ident.clone(),
            self.ty.clone(),
            self.value.clone(),
            ctx.errors,
        )?;

        Ok(())
    }
}

pub trait Ir0StorePattern: Sized {
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, usize)>>;

    fn get_idents(&self, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, Self)>>;
}

mod expr_pattern {
    prelude! {
        ir0::expr::{PatStructure, PatTuple, Pattern},
    }

    impl Ir0StorePattern for Pattern {
        fn store(&self, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, usize)>> {
            match self {
                Pattern::Identifier(name) => {
                    let id = ctx.ctx0.insert_ident(
                        name.clone(),
                        Scope::VeryLocal,
                        None,
                        true,
                        ctx.errors,
                    )?;
                    Ok(vec![(name.clone(), id)])
                }
                Pattern::Tuple(PatTuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(ctx))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(PatStructure { fields, .. }) => Ok(fields
                    .iter()
                    .map(|(field, optional_pattern)| {
                        if let Some(pattern) = optional_pattern {
                            pattern.store(ctx)
                        } else {
                            let id = ctx.ctx0.insert_local_ident(
                                field.clone(),
                                None,
                                true,
                                ctx.errors,
                            )?;
                            Ok(vec![(field.clone(), id)])
                        }
                    })
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Constant(_) | Pattern::Enumeration(_) | Pattern::Default(_) => Ok(vec![]),
            }
        }

        fn get_idents(&self, _ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, Pattern)>> {
            match self {
                Pattern::Identifier(name) => Ok(vec![(name.clone(), self.clone())]),
                Pattern::Tuple(PatTuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.get_idents(_ctx))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(PatStructure { fields, .. }) => Ok(fields
                    .iter()
                    .map(|(field, optional_pattern)| {
                        if let Some(pattern) = optional_pattern {
                            pattern.get_idents(_ctx)
                        } else {
                            Ok(vec![(field.clone(), Pattern::ident(field.clone()))])
                        }
                    })
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Constant(_) | Pattern::Enumeration(_) | Pattern::Default(_) => Ok(vec![]),
            }
        }
    }
}

pub trait Ir0StoreStmtPattern: Sized {
    fn store(&self, is_declaration: bool, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, usize)>>;

    fn store_inits(&self, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, usize)>>;

    fn get_idents(&self, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, Self)>>;
}

mod stmt_pattern {
    prelude! {
        ir0::stmt::{Pattern, Tuple, Typed},
    }

    impl Ir0StoreStmtPattern for Pattern {
        fn store(&self, is_declaration: bool, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, usize)>> {
            let loc = self.loc();

            match self {
                Pattern::Identifier(ident) => {
                    if is_declaration {
                        Err(error!(@ident.loc() => ErrorKind::expected_ty(ident.to_string())))
                    } else {
                        let id = ctx.ctx0.get_identifier_id(ident, false, ctx.errors)?;
                        // outputs should be already typed
                        let typ = ctx.get_typ(id).clone();
                        let id = ctx.ctx0.insert_local_ident(
                            ident.clone(),
                            Some(typ),
                            true,
                            ctx.errors,
                        )?;
                        Ok(vec![(ident.clone(), id)])
                    }
                    .dewrap(ctx.errors)
                }
                Pattern::Typed(Typed { ident, typ, .. }) => if is_declaration {
                    let typ = typ.clone().into_ir1(&mut ctx.add_loc(loc))?;
                    let id =
                        ctx.ctx0
                            .insert_local_ident(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(vec![(ident.clone(), id)])
                } else {
                    Err(error!(@ident.loc() => ErrorKind::re_ty(ident.to_string())))
                }
                .dewrap(ctx.errors),
                Pattern::Tuple(Tuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(is_declaration, ctx))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }

        fn store_inits(&self, ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, usize)>> {
            let loc = self.loc();

            match self {
                Pattern::Identifier(ident) => {
                    let id = ctx.ctx0.get_identifier_id(ident, false, ctx.errors)?;
                    // outputs should be already typed
                    let typ = ctx.get_typ(id).clone();
                    let id = ctx
                        .ctx0
                        .insert_init(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(vec![(ident.clone(), id)])
                }
                Pattern::Typed(Typed { ident, typ, .. }) => {
                    let typ = typ.clone().into_ir1(&mut ctx.add_loc(loc))?;
                    let id = ctx
                        .ctx0
                        .insert_init(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(vec![(ident.clone(), id)])
                }
                Pattern::Tuple(Tuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store_inits(ctx))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }

        fn get_idents(&self, _ctx: &mut ctx::Simple) -> TRes<Vec<(Ident, Pattern)>> {
            match self {
                Pattern::Identifier(ident) | Pattern::Typed(Typed { ident, .. }) => {
                    Ok(vec![(ident.clone(), self.clone())])
                }
                Pattern::Tuple(Tuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.get_idents(_ctx))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }
    }
}

impl Ir0Store for ir0::Typedef {
    /// Store typedef's identifiers in symbol table.
    fn store(&self, ctx: &mut ctx::Simple) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ctx.add_loc(loc);

        match self {
            ir0::Typedef::Structure { ident, fields, .. } => {
                ctx.local();

                let field_ids = fields
                    .iter()
                    .map(
                        |ir0::Colon {
                             left: ident,
                             right: typing,
                             ..
                         }| {
                            let typing = typing.clone().into_ir1(ctx)?;
                            let field_id = ctx.ctx0.insert_local_ident(
                                ident.clone(),
                                Some(typing),
                                true,
                                ctx.errors,
                            )?;
                            Ok(field_id)
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                ctx.global();

                let _ =
                    ctx.ctx0
                        .insert_struct(ident.clone(), field_ids.clone(), false, ctx.errors)?;
            }
            ir0::Typedef::Enumeration {
                ident, elements, ..
            } => {
                let element_ids = elements
                    .iter()
                    .map(|element_ident| {
                        let element_id = ctx.ctx0.insert_enum_elem(
                            element_ident.clone(),
                            ident.clone(),
                            false,
                            ctx.errors,
                        )?;
                        Ok(element_id)
                    })
                    .collect::<TRes<Vec<_>>>()?;

                let _ =
                    ctx.ctx0
                        .insert_enum(ident.clone(), element_ids.clone(), false, ctx.errors)?;
            }
            ir0::Typedef::Array {
                ident,
                size,
                array_type,
                ..
            } => {
                let typing = array_type.clone().into_ir1(ctx)?;
                let size = size.base10_parse().unwrap();
                let _ =
                    ctx.ctx0
                        .insert_array(ident.clone(), Some(typing), size, false, ctx.errors)?;
            }
        }

        Ok(())
    }
}

pub trait Ir0StoreEventPattern {
    /// Accumulates in `events_indices` the indices of events in the matched tuple.
    fn place_events(
        &self,
        events_indices: &mut HashMap<usize, usize>,
        idx: &mut usize,
        ctx: &mut ctx::Simple,
    ) -> TRes<()>;
    /// Creates event tuple and stores the events.
    fn create_tuple_pattern(
        self,
        tuple: &mut Vec<ir1::Pattern>,
        events_indices: &HashMap<usize, usize>,
        ctx: &mut ctx::Simple,
    ) -> TRes<Option<ir1::stream::Expr>>;
}

mod event_pattern {
    prelude! {
        ir0::equation::EventPattern,
    }

    impl Ir0StoreEventPattern for EventPattern {
        /// Accumulates in `events_indices` the indices of events in the matched tuple.
        fn place_events(
            &self,
            events_indices: &mut HashMap<usize, usize>,
            idx: &mut usize,
            ctx: &mut ctx::Simple,
        ) -> TRes<()> {
            match self {
                EventPattern::Tuple(tuple) => tuple
                    .patterns
                    .iter()
                    .try_for_each(|pattern| pattern.place_events(events_indices, idx, ctx)),
                EventPattern::Let(pattern) => {
                    let event_id = ctx
                        .ctx0
                        .get_identifier_id(&pattern.event, false, ctx.errors)?;
                    let _ = events_indices.entry(event_id).or_insert_with(|| {
                        let v = *idx;
                        *idx += 1;
                        v
                    });
                    Ok(())
                }
                EventPattern::RisingEdge(_) => Ok(()),
            }
        }

        /// Creates event tuple, stores the events and return rising_edges combined as a guard.
        fn create_tuple_pattern(
            self,
            tuple: &mut Vec<ir1::Pattern>,
            events_indices: &HashMap<usize, usize>,
            ctx: &mut ctx::Simple,
        ) -> TRes<Option<ir1::stream::Expr>> {
            match self {
                EventPattern::Tuple(patterns) => {
                    let mut guard: Option<ir1::stream::Expr> = None;
                    let mut combine_guard = |opt_guard: Option<ir1::stream::Expr>| {
                        if let Some(add_guard) = opt_guard {
                            if let Some(old_guard) = guard.take() {
                                guard = Some(ir1::stream::Expr::new(
                                    old_guard.loc(),
                                    ir1::stream::Kind::expr(ir1::expr::Kind::binop(
                                        BOp::And,
                                        old_guard,
                                        add_guard,
                                    )),
                                ));
                            } else {
                                guard = Some(add_guard);
                            }
                        }
                    };

                    patterns.patterns.into_iter().try_for_each(|pattern| {
                        let opt_guard = pattern.create_tuple_pattern(tuple, events_indices, ctx)?;
                        // combine all rising edge detections
                        combine_guard(opt_guard);
                        Ok(())
                    })?;

                    Ok(guard)
                }
                EventPattern::Let(pattern) => {
                    let loc = pattern.loc();
                    let ctx = &mut ctx.add_loc(loc);

                    // get the event identifier
                    let event_id = ctx
                        .ctx0
                        .get_identifier_id(&pattern.event, false, ctx.errors)?;

                    // transform inner_pattern into [ir1]
                    pattern.pattern.store(&mut ctx.rm_loc())?;
                    let inner_pattern = pattern.pattern.into_ir1(ctx)?;
                    let event_pattern = ir1::pattern::Pattern::new(
                        pattern.event.loc(),
                        ir1::pattern::Kind::present(event_id, inner_pattern),
                    );

                    // put event in tuple
                    let idx = events_indices[&event_id];
                    *tuple.get_mut(idx).unwrap() = event_pattern;

                    Ok(None)
                }
                EventPattern::RisingEdge(expr) => {
                    let loc = expr.loc();
                    let ctx = &mut ctx.add_pat_loc(None, loc);
                    let guard = ir1::stream::Kind::rising_edge(expr.into_ir1(ctx)?);
                    Ok(Some(ir1::stream::Expr::new(loc, guard)))
                }
            }
        }
    }
}
