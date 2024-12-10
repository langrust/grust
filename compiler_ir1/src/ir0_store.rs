prelude! {}

pub trait Ir0Store {
    fn store(&self, table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

mod component {
    prelude! {
        ir0::{Component, ComponentImport, Colon},
    }

    impl Ir0Store for Component {
        /// Store node's signals in symbol table.
        fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
            let location = Loc::mixed_site();
            let ctx = &mut ir1::ctx::WithLoc::new(location, symbol_table, errors);

            ctx.symbols.local();

            let name = self.ident.to_string();
            let period = self
                .period
                .as_ref()
                .map(|(_, literal, _)| literal.base10_parse().unwrap());
            let eventful = period.is_some()
                || self
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
                        let name = ident.to_string();
                        let typ = typ.clone().into_ir1(ctx)?;
                        let id = ctx.symbols.insert_signal(
                            name,
                            Scope::Input,
                            Some(typ),
                            true,
                            location.clone(),
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
                        let name = ident.to_string();
                        let typ = typ.clone().into_ir1(ctx)?;
                        let id = ctx.symbols.insert_signal(
                            name.clone(),
                            Scope::Output,
                            Some(typ),
                            true,
                            ctx.loc.clone(),
                            ctx.errors,
                        )?;
                        Ok((name, id))
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            // store locals and get their ids
            let locals = {
                let mut map = HashMap::with_capacity(25);
                for equation in self.equations.iter() {
                    equation.store_signals(false, &mut map, ctx.symbols, ctx.errors)?;
                }
                map.shrink_to_fit();
                map
            };

            ctx.symbols.global();

            let _ = ctx.symbols.insert_node(
                name,
                false,
                inputs,
                eventful,
                outputs,
                locals,
                period,
                ctx.loc.clone(),
                ctx.errors,
            )?;

            Ok(())
        }
    }

    impl Ir0Store for ComponentImport {
        /// Store node's signals in symbol table.
        fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
            let location = Loc::mixed_site();
            let ctx = &mut ir1::ctx::WithLoc::new(location, symbol_table, errors);
            ctx.symbols.local();

            let last = self.path.clone().segments.pop().unwrap().into_value();
            let name = last.ident.to_string();
            assert!(last.arguments.is_none());

            let period = self
                .period
                .as_ref()
                .map(|(_, literal, _)| literal.base10_parse().unwrap());

            let eventful = period.is_some()
                || self
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
                        let name = ident.to_string();
                        let typ = typ.clone().into_ir1(ctx)?;
                        let id = ctx.symbols.insert_signal(
                            name,
                            Scope::Input,
                            Some(typ),
                            true,
                            location.clone(),
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
                        let name = ident.to_string();
                        let typ = typ.clone().into_ir1(ctx)?;
                        let id = ctx.symbols.insert_signal(
                            name.clone(),
                            Scope::Output,
                            Some(typ),
                            true,
                            location.clone(),
                            ctx.errors,
                        )?;
                        Ok((name, id))
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            let locals = Default::default();

            symbol_table.global();

            let _ = symbol_table.insert_node(
                name, false, inputs, eventful, outputs, locals, period, location, errors,
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
        signals: &mut HashMap<String, usize>,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;

    /// Collects the identifiers of the equation from the symbol table.
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
    /// If the symbol table contains [ a -> id_a ] and [ b -> id_b ], with the above equations,
    /// the algorithm insert in the `signals` map [ a -> id_a ] and [ b -> id_b ].
    fn get_signals(
        &self,
        signals: &mut HashMap<String, ir0::stmt::Pattern>,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
}

mod equation {
    prelude! { ir0::equation::* }

    impl Ir0StoreSignals for Eq {
        fn store_signals(
            &self,
            store_outputs: bool,
            signals: &mut HashMap<String, usize>,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                // output definitions should be stored
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

        fn get_signals(
            &self,
            signals: &mut HashMap<String, ir0::stmt::Pattern>,
            symbol_table: &SymbolTable,
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
    }

    impl Ir0StoreSignals for ReactEq {
        fn store_signals(
            &self,
            store_outputs: bool,
            signals: &mut HashMap<String, usize>,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                // output definitions should be stored
                ReactEq::OutputDef(instantiation) if store_outputs => instantiation
                    .pattern
                    .store(false, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                // when output definitions are already stored (as component's outputs)
                ReactEq::OutputDef(_) => Ok(()),
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
                        // non-events are defined in all branches so we don't want them to trigger
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
                            signals.insert(k, v);
                            symbol_table.put_back_in_context(
                                v,
                                false,
                                Loc::mixed_site(),
                                errors,
                            )?;
                        }
                    }
                    Ok(())
                }
            }
        }

        fn get_signals(
            &self,
            signals: &mut HashMap<String, ir0::stmt::Pattern>,
            symbol_table: &SymbolTable,
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
            }
        }
    }
}

impl Ir0Store for Ast {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        self.items
            .iter()
            .map(|item| match item {
                ir0::Item::Component(component) => component.store(symbol_table, errors),
                ir0::Item::ComponentImport(component) => component.store(symbol_table, errors),
                ir0::Item::Function(function) => function.store(symbol_table, errors),
                ir0::Item::Typedef(typedef) => typedef.store(symbol_table, errors),
                ir0::Item::Service(_) | ir0::Item::Import(_) | ir0::Item::Export(_) => Ok(()),
            })
            .collect::<TRes<Vec<_>>>()?;
        Ok(())
    }
}

impl Ir0Store for ir0::Function {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let location = Loc::mixed_site();
        let ctx = &mut ir1::ctx::WithLoc::new(location, symbol_table, errors);
        ctx.symbols.local();

        let inputs = self
            .args
            .iter()
            .map(
                |ir0::Colon {
                     left: ident,
                     right: typ,
                     ..
                 }| {
                    let name = ident.to_string();
                    let typ = typ.clone().into_ir1(ctx)?;
                    let id = ctx.symbols.insert_identifier(
                        name.clone(),
                        Some(typ),
                        true,
                        location.clone(),
                        ctx.errors,
                    )?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        ctx.symbols.global();

        let _ = ctx.symbols.insert_function(
            self.ident.to_string(),
            inputs,
            None,
            false,
            ctx.loc.clone(),
            ctx.errors,
        )?;

        Ok(())
    }
}

pub trait Ir0StorePattern: Sized {
    fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(String, usize)>>;

    fn get_signals(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(String, Self)>>;
}

mod expr_pattern {
    prelude! {
        ir0::expr::{PatStructure, PatTuple, Pattern},
    }

    impl Ir0StorePattern for Pattern {
        fn store(
            &self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, usize)>> {
            let location = Loc::mixed_site();

            match self {
                Pattern::Identifier(name) => {
                    let id = symbol_table.insert_signal(
                        name.clone(),
                        Scope::VeryLocal,
                        None,
                        true,
                        location.clone(),
                        errors,
                    )?;
                    Ok(vec![(name.clone(), id)])
                }
                Pattern::Tuple(PatTuple { elements }) => Ok(elements
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
                                location.clone(),
                                errors,
                            )?;
                            Ok(vec![(field.clone(), id)])
                        }
                    })
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Constant(_) | Pattern::Enumeration(_) | Pattern::Default => Ok(vec![]),
            }
        }

        fn get_signals(
            &self,
            symbol_table: &SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, Pattern)>> {
            match self {
                Pattern::Identifier(name) => Ok(vec![(name.clone(), self.clone())]),
                Pattern::Tuple(PatTuple { elements }) => Ok(elements
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
                            Ok(vec![(field.clone(), Pattern::ident(field))])
                        }
                    })
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Constant(_) | Pattern::Enumeration(_) | Pattern::Default => Ok(vec![]),
            }
        }
    }
}

pub trait Ir0StoreStmtPattern: Sized {
    fn store(
        &self,
        is_declaration: bool,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(String, usize)>>;

    fn get_signals(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(String, Self)>>;

    fn into_default_expr(
        &self,
        defined_signals: &HashMap<String, usize>,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<ir1::stream::Expr>;
}

mod stmt_pattern {
    prelude! {
        ir0::stmt::{Pattern, Tuple, Typed},
    }

    impl Ir0StoreStmtPattern for Pattern {
        fn store(
            &self,
            is_declaration: bool,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, usize)>> {
            let location = Loc::mixed_site();

            match self {
                Pattern::Identifier(ident) => {
                    if is_declaration {
                        panic!("error in `Pattern`'s `store` for identifier `{}`", ident);
                        // Err(TerminationError)
                    } else {
                        let name = ident.to_string();
                        let id = symbol_table.get_identifier_id(
                            &name,
                            false,
                            location.clone(),
                            errors,
                        )?;
                        // outputs should be already typed
                        let typ = symbol_table.get_typ(id).clone();
                        let id = symbol_table.insert_identifier(
                            name.clone(),
                            Some(typ),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(vec![(name, id)])
                    }
                }
                Pattern::Typed(Typed { ident, typ, .. }) => {
                    if is_declaration {
                        let typ = typ.clone().into_ir1(&mut ir1::ctx::WithLoc::new(
                            location,
                            symbol_table,
                            errors,
                        ))?;
                        let id = symbol_table.insert_identifier(
                            ident.to_string(),
                            Some(typ),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(vec![(ident.to_string(), id)])
                    } else {
                        panic!(
                            "error in `Pattern`'s store for identifier `{}` with type `{}`",
                            ident, typ,
                        );
                        // Err(TerminationError)
                    }
                }
                Pattern::Tuple(Tuple { elements }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(is_declaration, symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }

        fn get_signals(
            &self,
            symbol_table: &SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, Pattern)>> {
            match self {
                Pattern::Identifier(ident) | Pattern::Typed(Typed { ident, .. }) => {
                    Ok(vec![(ident.to_string(), self.clone())])
                }
                Pattern::Tuple(Tuple { elements }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.get_signals(symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        }

        fn into_default_expr(
            &self,
            defined_signals: &HashMap<String, usize>,
            symbol_table: &SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<ir1::stream::Expr> {
            let kind = match self {
                Pattern::Identifier(ident) => {
                    let name = ident.to_string();
                    if let Some(id) = defined_signals.get(&name) {
                        ir1::stream::Kind::expr(ir1::expr::Kind::ident(*id))
                    } else {
                        let id = symbol_table.get_identifier_id(
                            &name,
                            false,
                            Loc::mixed_site(),
                            errors,
                        )?;
                        if symbol_table.get_typ(id).is_event() {
                            ir1::stream::Kind::none_event()
                        } else {
                            ir1::stream::Kind::fby(
                                id,
                                ir1::stream::expr(ir1::stream::Kind::expr(
                                    ir1::expr::Kind::constant(Constant::default()),
                                )),
                            )
                        }
                    }
                }
                Pattern::Typed(Typed { ident, typ, .. }) => {
                    let name = ident.to_string();
                    if let Some(id) = defined_signals.get(&name) {
                        ir1::stream::Kind::expr(ir1::expr::Kind::ident(*id))
                    } else {
                        let id = symbol_table.get_identifier_id(
                            &name,
                            false,
                            Loc::mixed_site(),
                            errors,
                        )?;
                        if typ.is_event() {
                            ir1::stream::Kind::none_event()
                        } else {
                            ir1::stream::Kind::fby(
                                id,
                                ir1::stream::expr(ir1::stream::Kind::expr(
                                    ir1::expr::Kind::constant(Constant::default()),
                                )),
                            )
                        }
                    }
                }
                Pattern::Tuple(Tuple { elements }) => {
                    let elements = elements
                        .iter()
                        .map(|pat| pat.into_default_expr(defined_signals, symbol_table, errors))
                        .collect::<TRes<_>>()?;
                    ir1::stream::Kind::expr(ir1::expr::Kind::tuple(elements))
                }
            };
            Ok(ir1::stream::expr(kind))
        }
    }
}

impl Ir0Store for ir0::Typedef {
    /// Store typedef's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let location = Loc::mixed_site();
        match self {
            ir0::Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                symbol_table.local();

                let field_ids = fields
                    .iter()
                    .map(|ir0::Colon { left: ident, .. }| {
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
                    .collect::<TRes<Vec<_>>>()?;

                symbol_table.global();

                let _ = symbol_table.insert_struct(
                    id.clone(),
                    field_ids.clone(),
                    false,
                    location.clone(),
                    errors,
                )?;
            }
            ir0::Typedef::Enumeration {
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
                    .collect::<TRes<Vec<_>>>()?;

                let _ = symbol_table.insert_enum(
                    id.clone(),
                    element_ids.clone(),
                    false,
                    location.clone(),
                    errors,
                )?;
            }
            ir0::Typedef::Array { ident, size, .. } => {
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

pub trait Ir0StoreEventPattern {
    /// Accumulates in `events_indices` the indices of events in the matched tuple.
    fn place_events(
        &self,
        events_indices: &mut HashMap<usize, usize>,
        idx: &mut usize,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
    /// Creates event tuple and stores the events.
    fn create_tuple_pattern(
        self,
        tuple: &mut Vec<ir1::Pattern>,
        events_indices: &HashMap<usize, usize>,
        symbol_table: &mut SymbolTable,
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
            symbol_table: &SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                EventPattern::Tuple(tuple) => tuple
                    .patterns
                    .iter()
                    .map(|pattern| pattern.place_events(events_indices, idx, symbol_table, errors))
                    .collect::<TRes<()>>(),
                EventPattern::Let(pattern) => {
                    let event_id = symbol_table.get_identifier_id(
                        &pattern.event.to_string(),
                        false,
                        Loc::mixed_site(),
                        errors,
                    )?;
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
            symbols: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Option<ir1::stream::Expr>> {
            match self {
                EventPattern::Tuple(patterns) => {
                    let mut guard = None;
                    let mut combine_guard = |opt_guard: Option<ir1::stream::Expr>| {
                        if let Some(add_guard) = opt_guard {
                            if let Some(old_guard) = guard.take() {
                                guard = Some(ir1::stream::expr(ir1::stream::Kind::expr(
                                    ir1::expr::Kind::binop(BOp::And, old_guard, add_guard),
                                )));
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
                    let location = Loc::mixed_site();
                    let ctx = &mut ir1::ctx::WithLoc::new(location, symbols, errors);

                    // get the event identifier
                    let event_id = ctx.symbols.get_identifier_id(
                        &pattern.event.to_string(),
                        false,
                        ctx.loc.clone(),
                        ctx.errors,
                    )?;

                    // transform inner_pattern into [ir1]
                    pattern.pattern.store(ctx.symbols, ctx.errors)?;
                    let inner_pattern = pattern.pattern.into_ir1(ctx)?;
                    let event_pattern =
                        ir1::pattern::init(ir1::pattern::Kind::present(event_id, inner_pattern));

                    // put event in tuple
                    let idx = events_indices[&event_id];
                    *tuple.get_mut(idx).unwrap() = event_pattern;

                    Ok(None)
                }
                EventPattern::RisingEdge(expr) => {
                    let location = Loc::mixed_site();
                    let ctx = &mut ir1::ctx::PatLoc::new(None, location, symbols, errors);
                    let guard = ir1::stream::Kind::rising_edge(expr.into_ir1(ctx)?);
                    Ok(Some(ir1::stream::expr(guard)))
                }
            }
        }
    }
}
