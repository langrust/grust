prelude! {}

pub trait Ir0Store {
    fn store(&self, table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()>;
}

mod component {
    prelude! {
        ir0::{Component, ComponentImport, Colon},
    }

    impl Ir0Store for Component {
        /// Store node's signals in symbol table.
        fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
            let loc = self.loc();
            let ctx = &mut ir1::ctx::WithLoc::new(loc, symbol_table, errors);

            ctx.local();

            let eventful = self
                    .args
                    .iter()
                    .any(|Colon { right: typ, .. }| typ.is_event());

            // store input signals and get their ids
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
                        let id = ctx.ctx0.insert_signal(
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
                        let id = ctx.ctx0.insert_signal(
                            ident.clone(),
                            Scope::Output,
                            Some(typ),
                            true,
                            ctx.errors,
                        )?;
                        Ok((self.ident.clone(), id))
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            // store locals and get their ids
            let locals = {
                let mut locals = HashMap::with_capacity(25);
                for equation in self.equations.iter() {
                    equation.store_signals(false, &mut locals, ctx.ctx0, ctx.errors)?;
                }
                locals.shrink_to_fit();
                locals
            };

            // store inits and get their ids
            let inits = {
                let mut inits = HashMap::with_capacity(5);
                for equation in self.equations.iter() {
                    equation.store_inits(&mut inits, ctx.ctx0, ctx.errors)?;
                }
                inits.shrink_to_fit();
                inits
            };

            ctx.global();

            let _ = ctx.ctx0.insert_node(
                self.ident.clone(),
                false,
                inputs,
                eventful,
                outputs,
                locals,
                inits,
                ctx.errors,
            )?;

            Ok(())
        }
    }

    impl Ir0Store for ComponentImport {
        /// Store node's signals in symbol table.
        fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
            let loc = self.loc();
            let ctx = &mut ir1::ctx::WithLoc::new(loc, symbol_table, errors);
            ctx.local();

            let last = self.path.clone().segments.pop().unwrap().into_value();
            assert!(last.arguments.is_none());

            let eventful = self
                    .args
                    .iter()
                    .any(|Colon { right: typ, .. }| typ.is_event());

            // store input signals and get their ids
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
                        let id = ctx.ctx0.insert_signal(
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
                        let id = ctx.ctx0.insert_signal(
                            ident.clone(),
                            Scope::Output,
                            Some(typ),
                            true,
                            ctx.errors,
                        )?;
                        Ok((last.ident.clone(), id))
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            let locals = Default::default();
            let inits = Default::default();

            symbol_table.global();

            let _ = symbol_table.insert_node(
                last.ident.clone(),
                false,
                inputs,
                eventful,
                outputs,
                locals,
                inits,
                errors,
            )?;

            Ok(())
        }
    }
}

pub trait Ir0StoreSignals {
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
    /// With the above equations, the algorithm insert in the `signals` map
    /// [ a -> id_a ] and [ b -> id_b ].
    fn store_signals(
        &self,
        store_outputs: bool,
        signals: &mut HashMap<Ident, usize>,
        symbol_table: &mut Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;

    fn store_inits(
        &self,
        inits: &mut HashMap<Ident, usize>,
        symbol_table: &mut Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;

    /// Collects the identifiers of the equation.
    fn get_signals(
        &self,
        signals: &mut HashMap<Ident, ir0::stmt::Pattern>,
        symbol_table: &Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;

    /// Collects the initializations.
    fn get_inits(
        &self,
        inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
        symbol_table: &Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
}

pub trait Ir0StoreInit: Sized {
    fn store_inits(
        &self,
        inits: &mut HashMap<Ident, usize>,
        symbol_table: &mut Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;

    fn get_inits(
        &self,
        inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
        symbol_table: &Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
}

mod equation {
    prelude! { ir0::equation::* }

    impl Ir0StoreSignals for Eq {
        fn store_signals(
            &self,
            store_outputs: bool,
            signals: &mut HashMap<Ident, usize>,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                // when output definitions should be stored
                Eq::OutputDef(instantiation) if store_outputs => instantiation
                    .pattern
                    .store(false, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                // when output definitions are already stored (as component outputs)
                Eq::OutputDef(_) => Ok(()),
                Eq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .store(true, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                Eq::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations.iter() {
                        eq.store_signals(store_outputs, signals, symbol_table, errors)?;
                    }
                    Ok(())
                }
            }
        }

        fn store_inits(
            &self,
            _inits: &mut HashMap<Ident, usize>,
            _symbol_table: &mut Ctx,
            _errors: &mut Vec<Error>,
        ) -> TRes<()> {
            Ok(())
        }

        fn get_signals(
            &self,
            signals: &mut HashMap<Ident, ir0::stmt::Pattern>,
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                Eq::OutputDef(instantiation) => instantiation
                    .pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                Eq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                Eq::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations {
                        eq.get_signals(signals, symbol_table, errors)?;
                    }
                    Ok(())
                }
            }
        }

        fn get_inits(
            &self,
            _inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
            _symbol_table: &Ctx,
            _errors: &mut Vec<Error>,
        ) -> TRes<()> {
            Ok(())
        }
    }

    impl Ir0StoreInit for InitArmWhen {
        fn store_inits(
            &self,
            inits: &mut HashMap<Ident, usize>,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            for init in &self.equations {
                let idents = init.pattern.store_inits(symbol_table, errors)?;
                inits.extend(idents);
            }
            Ok(())
        }

        fn get_inits(
            &self,
            inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            for init in &self.equations {
                let idents = init.pattern.get_signals(symbol_table, errors)?;
                inits.extend(idents);
            }
            Ok(())
        }
    }

    impl Ir0StoreSignals for ReactEq {
        fn store_signals(
            &self,
            store_outputs: bool,
            signals: &mut HashMap<Ident, usize>,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                // output definitions should be stored
                ReactEq::OutputDef(instantiation) if store_outputs => instantiation
                    .pattern
                    .store(false, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                // when output definitions are already stored (as component's outputs)
                ReactEq::OutputDef(_) | ReactEq::Init(_) => Ok(()),
                ReactEq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .store(true, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                ReactEq::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations.iter() {
                        eq.store_signals(store_outputs, signals, symbol_table, errors)?;
                    }
                    Ok(())
                }
                ReactEq::MatchWhen(MatchWhen { arms, .. }) => {
                    // we want to collect every identifier, but events might be declared in only one
                    // branch then, it is needed to explore all branches
                    let mut when_signals = HashMap::new();
                    let mut add_signals = |equations: &Vec<Eq>| {
                        // some flows are defined in multiple branches so we don't want them to trigger
                        // the *duplicated definition* error.
                        symbol_table.local();
                        for eq in equations {
                            eq.store_signals(
                                store_outputs,
                                &mut when_signals,
                                symbol_table,
                                errors,
                            )?;
                        }
                        symbol_table.global();
                        Ok(())
                    };
                    for EventArmWhen { equations, .. } in arms {
                        add_signals(equations)?
                    }
                    // put the identifiers back in context
                    for (k, v) in when_signals.into_iter() {
                        if signals.contains_key(&k) {
                            // todo: delete the symbol
                        } else {
                            let loc = k.loc();
                            signals.insert(k, v);
                            symbol_table.put_back_in_context(v, false, loc, errors)?;
                        }
                    }
                    Ok(())
                }
            }
        }

        fn store_inits(
            &self,
            inits: &mut HashMap<Ident, usize>,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                ReactEq::Init(init) => init
                    .pattern
                    .store_inits(symbol_table, errors)
                    .map(|idents| inits.extend(idents)),
                ReactEq::OutputDef(instantiation) => {
                    if let ir0::stream::ReactExpr::When(ir0::stream::When {
                        init: Some(_), ..
                    }) = instantiation.expr
                    {
                        instantiation
                            .pattern
                            .store_inits(symbol_table, errors)
                            .map(|idents| inits.extend(idents))?;
                    }
                    Ok(())
                }
                ReactEq::LocalDef(declaration) => {
                    if let ir0::stream::ReactExpr::When(ir0::stream::When {
                        init: Some(_), ..
                    }) = declaration.expr
                    {
                        declaration
                            .typed_pattern
                            .store_inits(symbol_table, errors)
                            .map(|idents| inits.extend(idents))?;
                    }
                    Ok(())
                }
                ReactEq::Match(Match { .. }) => Ok(()),
                ReactEq::MatchWhen(MatchWhen { init, .. }) => {
                    // store initializations
                    if let Some(init) = init {
                        init.store_inits(inits, symbol_table, errors)?;
                    }
                    Ok(())
                }
            }
        }

        fn get_signals(
            &self,
            signals: &mut HashMap<Ident, ir0::stmt::Pattern>,
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                ReactEq::OutputDef(instantiation) => instantiation
                    .pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                ReactEq::LocalDef(declaration) => declaration
                    .typed_pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                ReactEq::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations {
                        eq.get_signals(signals, symbol_table, errors)?;
                    }
                    Ok(())
                }
                ReactEq::MatchWhen(MatchWhen { arms, .. }) => {
                    let mut add_signals = |equations: &Vec<Eq>| {
                        // we want to collect every identifier, but events might be declared in only
                        // one branch then, it is needed to explore all branches
                        for eq in equations {
                            eq.get_signals(signals, symbol_table, errors)?;
                        }
                        Ok(())
                    };
                    for EventArmWhen { equations, .. } in arms {
                        add_signals(equations)?
                    }
                    Ok(())
                }
                ReactEq::Init(_) => Ok(()),
            }
        }

        fn get_inits(
            &self,
            inits: &mut HashMap<Ident, ir0::stmt::Pattern>,
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                ReactEq::OutputDef(_) | ReactEq::LocalDef(_) | ReactEq::Match(_) => Ok(()),
                ReactEq::MatchWhen(MatchWhen { init, .. }) => {
                    if let Some(init) = init {
                        init.get_inits(inits, symbol_table, errors)
                    } else {
                        Ok(())
                    }
                }
                ReactEq::Init(init) => init
                    .pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| inits.extend(idents)),
            }
        }
    }
}

impl Ir0Store for Ast {
    fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        self.items
            .iter()
            .map(|item| match item {
                ir0::Item::Component(component) => component.store(symbol_table, errors),
                ir0::Item::ComponentImport(component) => component.store(symbol_table, errors),
                ir0::Item::Function(function) => function.store(symbol_table, errors),
                ir0::Item::Typedef(typedef) => typedef.store(symbol_table, errors),
                ir0::Item::ExtFun(extfun) => extfun.store(symbol_table, errors),
                ir0::Item::Service(_) | ir0::Item::Import(_) | ir0::Item::Export(_) => Ok(()),
            })
            .collect::<TRes<Vec<_>>>()?;
        Ok(())
    }
}

impl Ir0Store for ir0::Function {
    fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ir1::ctx::WithLoc::new(loc, symbol_table, errors);
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
                            .insert_identifier(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        ctx.global();

        let _ =
            ctx.ctx0
                .insert_function(self.ident.clone(), inputs, None, false, None, ctx.errors)?;

        Ok(())
    }
}

impl Ir0Store for ir0::ExtFunDecl {
    fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        let loc = self.loc();
        let ctx = &mut ir1::ctx::WithLoc::new(loc, symbol_table, errors);
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
                            .insert_identifier(ident.clone(), Some(typ), true, ctx.errors)?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        ctx.global();

        let _ = ctx.ctx0.insert_function(
            self.ident.clone(),
            inputs,
            None,
            false,
            Some(self.path.clone()),
            ctx.errors,
        )?;

        Ok(())
    }
}

pub trait Ir0StorePattern: Sized {
    fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<Vec<(Ident, usize)>>;

    fn get_signals(&self, symbol_table: &Ctx, errors: &mut Vec<Error>) -> TRes<Vec<(Ident, Self)>>;
}

mod expr_pattern {
    prelude! {
        ir0::expr::{PatStructure, PatTuple, Pattern},
    }

    impl Ir0StorePattern for Pattern {
        fn store(
            &self,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(Ident, usize)>> {
            match self {
                Pattern::Identifier(name) => {
                    let id = symbol_table.insert_signal(
                        name.clone(),
                        Scope::VeryLocal,
                        None,
                        true,
                        errors,
                    )?;
                    Ok(vec![(name.clone(), id)])
                }
                Pattern::Tuple(PatTuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(PatStructure { fields, .. }) => Ok(fields
                    .iter()
                    .map(|(field, optional_pattern)| {
                        if let Some(pattern) = optional_pattern {
                            pattern.store(symbol_table, errors)
                        } else {
                            let id = symbol_table.insert_identifier(
                                field.clone(),
                                None,
                                true,
                                errors,
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

        fn get_signals(
            &self,
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(Ident, Pattern)>> {
            match self {
                Pattern::Identifier(name) => Ok(vec![(name.clone(), self.clone())]),
                Pattern::Tuple(PatTuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.get_signals(symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(PatStructure { fields, .. }) => Ok(fields
                    .iter()
                    .map(|(field, optional_pattern)| {
                        if let Some(pattern) = optional_pattern {
                            pattern.get_signals(symbol_table, errors)
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
    fn store(
        &self,
        is_declaration: bool,
        symbol_table: &mut Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(Ident, usize)>>;

    fn store_inits(
        &self,
        symbol_table: &mut Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(Ident, usize)>>;

    fn get_signals(&self, symbol_table: &Ctx, errors: &mut Vec<Error>) -> TRes<Vec<(Ident, Self)>>;
}

mod stmt_pattern {
    prelude! {
        ir0::stmt::{Pattern, Tuple, Typed},
    }

    impl Ir0StoreStmtPattern for Pattern {
        fn store(
            &self,
            is_declaration: bool,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(Ident, usize)>> {
            let loc = self.loc();

            match self {
                Pattern::Identifier(ident) => {
                    if is_declaration {
                        panic!("error in `Pattern`'s `store` for identifier `{}`", ident);
                        // Err(TerminationError)
                    } else {
                        let id = symbol_table.get_identifier_id(ident, false, errors)?;
                        // outputs should be already typed
                        let typ = symbol_table.get_typ(id).clone();
                        let id = symbol_table.insert_identifier(
                            ident.clone(),
                            Some(typ),
                            true,
                            errors,
                        )?;
                        Ok(vec![(ident.clone(), id)])
                    }
                }
                Pattern::Typed(Typed { ident, typ, .. }) => {
                    if is_declaration {
                        let typ = typ.clone().into_ir1(&mut ir1::ctx::WithLoc::new(
                            loc,
                            symbol_table,
                            errors,
                        ))?;
                        let id = symbol_table.insert_identifier(
                            ident.clone(),
                            Some(typ),
                            true,
                            errors,
                        )?;
                        Ok(vec![(ident.clone(), id)])
                    } else {
                        panic!(
                            "error in `Pattern`'s store for identifier `{}` with type `{}`",
                            ident, typ,
                        );
                        // Err(TerminationError)
                    }
                }
                Pattern::Tuple(Tuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(is_declaration, symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }

        fn store_inits(
            &self,
            symbol_table: &mut Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(Ident, usize)>> {
            let loc = self.loc();

            match self {
                Pattern::Identifier(ident) => {
                    let id = symbol_table.get_identifier_id(ident, false, errors)?;
                    // outputs should be already typed
                    let typ = symbol_table.get_typ(id).clone();
                    let id = symbol_table.insert_init(ident.clone(), Some(typ), true, errors)?;
                    Ok(vec![(ident.clone(), id)])
                }
                Pattern::Typed(Typed { ident, typ, .. }) => {
                    let typ = typ.clone().into_ir1(&mut ir1::ctx::WithLoc::new(
                        loc,
                        symbol_table,
                        errors,
                    ))?;
                    let id = symbol_table.insert_init(ident.clone(), Some(typ), true, errors)?;
                    Ok(vec![(ident.clone(), id)])
                }
                Pattern::Tuple(Tuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store_inits(symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }

        fn get_signals(
            &self,
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(Ident, Pattern)>> {
            match self {
                Pattern::Identifier(ident) | Pattern::Typed(Typed { ident, .. }) => {
                    Ok(vec![(ident.clone(), self.clone())])
                }
                Pattern::Tuple(Tuple { elements, .. }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.get_signals(symbol_table, errors))
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
    fn store(&self, symbol_table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        match self {
            ir0::Typedef::Structure { ident, fields, .. } => {
                symbol_table.local();

                let field_ids = fields
                    .iter()
                    .map(|ir0::Colon { left: ident, .. }| {
                        let field_id =
                            symbol_table.insert_identifier(ident.clone(), None, true, errors)?;
                        Ok(field_id)
                    })
                    .collect::<TRes<Vec<_>>>()?;

                symbol_table.global();

                let _ =
                    symbol_table.insert_struct(ident.clone(), field_ids.clone(), false, errors)?;
            }
            ir0::Typedef::Enumeration {
                ident, elements, ..
            } => {
                let element_ids = elements
                    .iter()
                    .map(|element_ident| {
                        let element_id = symbol_table.insert_enum_elem(
                            element_ident.clone(),
                            ident.clone(),
                            false,
                            errors,
                        )?;
                        Ok(element_id)
                    })
                    .collect::<TRes<Vec<_>>>()?;

                let _ =
                    symbol_table.insert_enum(ident.clone(), element_ids.clone(), false, errors)?;
            }
            ir0::Typedef::Array { ident, size, .. } => {
                let size = size.base10_parse().unwrap();
                let _ = symbol_table.insert_array(ident.clone(), None, size, false, errors)?;
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
        symbol_table: &Ctx,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
    /// Creates event tuple and stores the events.
    fn create_tuple_pattern(
        self,
        tuple: &mut Vec<ir1::Pattern>,
        events_indices: &HashMap<usize, usize>,
        symbol_table: &mut Ctx,
        errors: &mut Vec<Error>,
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
            symbol_table: &Ctx,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                EventPattern::Tuple(tuple) => tuple
                    .patterns
                    .iter()
                    .map(|pattern| pattern.place_events(events_indices, idx, symbol_table, errors))
                    .collect::<TRes<()>>(),
                EventPattern::Let(pattern) => {
                    let event_id = symbol_table.get_identifier_id(&pattern.event, false, errors)?;
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
            symbols: &mut Ctx,
            errors: &mut Vec<Error>,
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

                    patterns
                        .patterns
                        .into_iter()
                        .map(|pattern| {
                            let opt_guard = pattern.create_tuple_pattern(
                                tuple,
                                events_indices,
                                symbols,
                                errors,
                            )?;
                            // combine all rising edge detections
                            combine_guard(opt_guard);
                            Ok(())
                        })
                        .collect::<TRes<()>>()?;

                    Ok(guard)
                }
                EventPattern::Let(pattern) => {
                    let loc = pattern.loc();
                    let ctx = &mut ir1::ctx::WithLoc::new(loc, symbols, errors);

                    // get the event identifier
                    let event_id = ctx
                        .ctx0
                        .get_identifier_id(&pattern.event, false, ctx.errors)?;

                    // transform inner_pattern into [ir1]
                    pattern.pattern.store(ctx.ctx0, ctx.errors)?;
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
                    let ctx = &mut ir1::ctx::PatLoc::new(None, loc, symbols, errors);
                    let guard = ir1::stream::Kind::rising_edge(expr.into_ir1(ctx)?);
                    Ok(Some(ir1::stream::Expr::new(loc, guard)))
                }
            }
        }
    }
}
