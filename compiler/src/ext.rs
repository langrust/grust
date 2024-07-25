prelude! {}

pub mod hir_ext;

pub trait ComponentExt {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

mod component {
    prelude! {
        ast::{Component, Colon}
    }

    impl super::ComponentExt for Component {
        /// Store node's signals in symbol table.
        fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
            symbol_table.local();

            let name = self.ident.to_string();
            let period = self
                .period
                .as_ref()
                .map(|(_, literal, _)| literal.base10_parse().unwrap());
            let eventful = period.is_some()
                || self
                    .args
                    .iter()
                    .any(|Colon { right: typing, .. }| typing.is_event());
            let location = Location::default();

            // store input signals and get their ids
            let inputs = self
                .args
                .iter()
                .map(
                    |Colon {
                         left: ident,
                         right: typing,
                         ..
                     }| {
                        let name = ident.to_string();
                        let typing =
                            typing
                                .clone()
                                .hir_from_ast(&location, symbol_table, errors)?;
                        let id = symbol_table.insert_signal(
                            name,
                            Scope::Input,
                            Some(typing),
                            true,
                            location.clone(),
                            errors,
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
                         right: typing,
                         ..
                     }| {
                        let name = ident.to_string();
                        let typing =
                            typing
                                .clone()
                                .hir_from_ast(&location, symbol_table, errors)?;
                        let id = symbol_table.insert_signal(
                            name.clone(),
                            Scope::Output,
                            Some(typing),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok((name, id))
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            // store locals and get their ids
            let locals = {
                let mut map = HashMap::with_capacity(25);
                for equation in self.equations.iter() {
                    equation.store_signals(false, &mut map, symbol_table, errors)?;
                }
                map.shrink_to_fit();
                map
            };

            symbol_table.global();

            let _ = symbol_table.insert_node(
                name, false, inputs, eventful, outputs, locals, period, location, errors,
            )?;

            Ok(())
        }
    }
}

pub trait EquationExt {
    /// Creates identifiers for the equation (depending on the config `even_outputs`)
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
        even_outputs: bool,
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
        signals: &mut HashMap<String, ast::Pattern>,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
}

mod equation {
    prelude! { ast::equation::* }

    impl super::EquationExt for Equation {
        fn store_signals(
            &self,
            even_outputs: bool,
            signals: &mut HashMap<String, usize>,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                // output defintions should be stored
                Equation::OutputDef(instantiation) if even_outputs => instantiation
                    .pattern
                    .store(false, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                // when output defintions are already stored (as component's outputs)
                Equation::OutputDef(_) => Ok(()),
                Equation::LocalDef(declaration) => declaration
                    .typed_pattern
                    .store(true, symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                Equation::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations.iter() {
                        eq.store_signals(even_outputs, signals, symbol_table, errors)?;
                    }
                    Ok(())
                }
                Equation::MatchWhen(MatchWhen { arms, default, .. }) => {
                    // we want to collect every identifier, but events might be declared in only one branch
                    // then, it is needed to explore all branches
                    let mut when_signals = HashMap::new();
                    let mut add_signals = |equations: &Vec<Equation>| {
                        // non-events are defined in all branches so we don't want them to trigger
                        // the 'duplicated definiiton' error.
                        symbol_table.local();
                        for eq in equations {
                            eq.store_signals(
                                even_outputs,
                                &mut when_signals,
                                symbol_table,
                                errors,
                            )?;
                        }
                        symbol_table.global();
                        Ok(())
                    };
                    if let Some(DefaultArmWhen { equations, .. }) = default {
                        add_signals(equations)?
                    }
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
                                Location::default(),
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
            signals: &mut HashMap<String, ast::Pattern>,
            symbol_table: &SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match self {
                Equation::OutputDef(instantiation) => instantiation
                    .pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                Equation::LocalDef(declaration) => declaration
                    .typed_pattern
                    .get_signals(symbol_table, errors)
                    .map(|idents| signals.extend(idents)),
                Equation::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    for eq in equations {
                        eq.get_signals(signals, symbol_table, errors)?;
                    }
                    Ok(())
                }
                Equation::MatchWhen(MatchWhen { arms, default, .. }) => {
                    let mut add_signals = |equations: &Vec<Equation>| {
                        // we want to collect every identifier, but events might be declared in only one branch
                        // then, it is needed to explore all branches
                        for eq in equations {
                            eq.get_signals(signals, symbol_table, errors)?;
                        }
                        Ok(())
                    };
                    if let Some(DefaultArmWhen { equations, .. }) = default {
                        add_signals(equations)?
                    }
                    for EventArmWhen { equations, .. } in arms {
                        add_signals(equations)?
                    }
                    Ok(())
                }
            }
        }
    }
}

pub trait AstExt {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

impl AstExt for Ast {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        self.items
            .iter()
            .map(|item| match item {
                crate::ast::Item::Component(component) => component.store(symbol_table, errors),
                crate::ast::Item::Function(function) => function.store(symbol_table, errors),
                crate::ast::Item::Typedef(typedef) => typedef.store(symbol_table, errors),
                crate::ast::Item::Service(_)
                | crate::ast::Item::Import(_)
                | crate::ast::Item::Export(_) => Ok(()),
            })
            .collect::<TRes<Vec<_>>>()?;
        Ok(())
    }
}

pub trait FunctionExt {
    /// Store function's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

impl FunctionExt for ast::Function {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        symbol_table.local();

        let location = Location::default();
        let inputs = self
            .args
            .iter()
            .map(
                |ast::Colon {
                     left: ident,
                     right: typing,
                     ..
                 }| {
                    let name = ident.to_string();
                    let typing = typing
                        .clone()
                        .hir_from_ast(&location, symbol_table, errors)?;
                    let id = symbol_table.insert_identifier(
                        name.clone(),
                        Some(typing),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        symbol_table.global();

        let _ = symbol_table.insert_function(
            self.ident.to_string(),
            inputs,
            None,
            false,
            location,
            errors,
        )?;

        Ok(())
    }
}

pub trait TypExt {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ>;
}

mod typ {
    use syn::{
        punctuated::{Pair, Punctuated},
        Token,
    };

    prelude! { ast::Typ }

    impl TypExt for Typ {
        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(
            self,
            location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Typ> {
            // precondition: Typedefs are stored in symbol table
            // postcondition: construct a new Type without `Typ::NotDefinedYet`
            match self {
                Typ::Array { bracket_token, ty, semi_token, size } => Ok(Typ::Array {
                    bracket_token,
                    ty: Box::new(ty.hir_from_ast(location, symbol_table, errors)?),
                    semi_token,
                    size
                }),
                Typ::Tuple { paren_token, elements } => Ok(Typ::Tuple {
                    paren_token,
                    elements: elements.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.hir_from_ast(location, symbol_table, errors)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?
                }),
                Typ::NotDefinedYet(name) => symbol_table
                    .get_struct_id(&name.to_string(), false, location.clone(), &mut vec![])
                    .map(|id| Typ::Structure { name: name.clone(), id })
                    .or_else(|_| {
                        symbol_table
                            .get_enum_id(&name.to_string(), false, location.clone(), &mut vec![])
                            .map(|id| Typ::Enumeration { name: name.clone(), id })
                    })
                    .or_else(|_| {
                        let id = symbol_table.get_array_id(&name.to_string(), false, location.clone(), errors)?;
                        Ok(symbol_table.get_array(id))
                    }),
                Typ::Abstract { paren_token, inputs, arrow_token, output } => {
                    let inputs = inputs.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.hir_from_ast(location, symbol_table, errors)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?;
                    let output = output.hir_from_ast(location, symbol_table, errors)?;
                    Ok(Typ::Abstract { paren_token, inputs, arrow_token, output: output.into() })
                }
                Typ::SMEvent { ty, question_token } => Ok(Typ::SMEvent {
                    ty: Box::new(ty.hir_from_ast(location, symbol_table, errors)?),
                    question_token
                }),
                Typ::Signal { signal_token, ty } => Ok(Typ::Signal {
                    signal_token,
                    ty: Box::new(ty.hir_from_ast(location, symbol_table, errors)?),
                }),
                Typ::Event { event_token, ty } => Ok(Typ::Event {
                    event_token,
                    ty: Box::new(ty.hir_from_ast(location, symbol_table, errors)?),
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
}

pub trait PatternExt: Sized {
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
}

mod pattern {
    prelude! {
        ast::pattern::*,
    }

    impl super::PatternExt for Pattern {
        fn store(
            &self,
            is_declaration: bool,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, usize)>> {
            let location = Location::default();

            match self {
                Pattern::Identifier(name) => {
                    if is_declaration {
                        let id = symbol_table.insert_identifier(
                            name.clone(),
                            None,
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(vec![(name.clone(), id)])
                    } else {
                        let id = symbol_table.get_identifier_id(
                            name,
                            false,
                            location.clone(),
                            errors,
                        )?;
                        // outputs should be already typed
                        let typing = symbol_table.get_type(id).clone();
                        let id = symbol_table.insert_identifier(
                            name.clone(),
                            Some(typing),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(vec![(name.clone(), id)])
                    }
                }
                Pattern::Typed(Typed { pattern, .. }) => {
                    pattern.store(is_declaration, symbol_table, errors)
                }
                Pattern::Tuple(Tuple { elements }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(is_declaration, symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(Structure { fields, .. }) => Ok(fields
                    .iter()
                    .map(|(field, optional_pattern)| {
                        if let Some(pattern) = optional_pattern {
                            pattern.store(is_declaration, symbol_table, errors)
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
                Pattern::Typed(Typed { pattern, .. }) => match pattern.as_ref() {
                    Pattern::Identifier(name) => Ok(vec![(name.clone(), self.clone())]),
                    _ => todo!("not supported"),
                },
                Pattern::Tuple(Tuple { elements }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.get_signals(symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(Structure { fields, .. }) => Ok(fields
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

pub trait TypedefExt {
    /// Store typedef's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

impl TypedefExt for ast::Typedef {
    /// Store typedef's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let location = Location::default();
        match self {
            ast::Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                symbol_table.local();

                let field_ids = fields
                    .iter()
                    .map(|ast::Colon { left: ident, .. }| {
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
            ast::Typedef::Enumeration {
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
            ast::Typedef::Array { ident, size, .. } => {
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

pub trait StreamExprExt {
    fn check_is_constant(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
}

mod stream_expr {
    prelude! {
        ast::{ expr::{Application, Array, Binop, IfThenElse, Structure, Tuple, Unop}, stream },
    }

    impl StreamExprExt for ast::stream::Expr {
        fn check_is_constant(
            &self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match &self {
                // Constant by default
                stream::Expr::Constant { .. } | stream::Expr::Enumeration { .. } => Ok(()),
                // Not constant by default
                stream::Expr::TypedAbstraction { .. }
                | stream::Expr::Match { .. }
                | stream::Expr::When { .. }
                | stream::Expr::FieldAccess { .. }
                | stream::Expr::TupleElementAccess { .. }
                | stream::Expr::Map { .. }
                | stream::Expr::Fold { .. }
                | stream::Expr::Sort { .. }
                | stream::Expr::Zip { .. }
                | stream::Expr::Fby { .. } => {
                    let error = Error::ExpectConstant {
                        location: Location::default(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
                // It depends
                stream::Expr::Identifier(id) => {
                    // check id exists
                    let id = symbol_table
                        .get_identifier_id(&id, false, Location::default(), &mut vec![])
                        .or_else(|_| {
                            symbol_table.get_function_id(&id, false, Location::default(), errors)
                        })?;
                    // check it is a function or and operator
                    if symbol_table.is_function(id) {
                        Ok(())
                    } else {
                        let error = Error::ExpectConstant {
                            location: Location::default(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
                stream::Expr::Unop(Unop { expression, .. }) => {
                    expression.check_is_constant(symbol_table, errors)
                }
                stream::Expr::Binop(Binop {
                    left_expression,
                    right_expression,
                    ..
                }) => {
                    left_expression.check_is_constant(symbol_table, errors)?;
                    right_expression.check_is_constant(symbol_table, errors)
                }
                stream::Expr::IfThenElse(IfThenElse {
                    expression,
                    true_expression,
                    false_expression,
                    ..
                }) => {
                    expression.check_is_constant(symbol_table, errors)?;
                    true_expression.check_is_constant(symbol_table, errors)?;
                    false_expression.check_is_constant(symbol_table, errors)
                }
                stream::Expr::Application(Application {
                    function_expression,
                    inputs,
                }) => {
                    function_expression.check_is_constant(symbol_table, errors)?;
                    inputs
                        .iter()
                        .map(|expression| expression.check_is_constant(symbol_table, errors))
                        .collect::<TRes<_>>()
                }
                stream::Expr::Structure(Structure { fields, .. }) => fields
                    .iter()
                    .map(|(_, expression)| expression.check_is_constant(symbol_table, errors))
                    .collect::<TRes<_>>(),
                stream::Expr::Array(Array { elements })
                | stream::Expr::Tuple(Tuple { elements }) => elements
                    .iter()
                    .map(|expression| expression.check_is_constant(symbol_table, errors))
                    .collect::<TRes<_>>(),
            }
        }
    }
}

pub trait EventPatternExt {
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
        tuple: &mut Vec<hir::Pattern>,
        events_indices: &HashMap<usize, usize>,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Option<ast::stream::Expr>>;
}

mod event_pattern {
    prelude! {
        ast::equation::EventPattern
    }

    impl super::EventPatternExt for EventPattern {
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
                        Location::default(),
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

        /// Creates event tuple and stores the events.
        fn create_tuple_pattern(
            self,
            tuple: &mut Vec<hir::Pattern>,
            events_indices: &HashMap<usize, usize>,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Option<ast::stream::Expr>> {
            match self {
                EventPattern::Tuple(patterns) => {
                    let mut guard = None;
                    let mut combine_guard = |opt_guard: Option<ast::stream::Expr>| {
                        if let Some(add_guard) = opt_guard {
                            if let Some(old_guard) = guard.take() {
                                guard = Some(ast::stream::Expr::binop(ast::expr::Binop::new(
                                    operator::BinaryOperator::And,
                                    old_guard,
                                    add_guard,
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
                                symbol_table,
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
                    // get the event identifier
                    let event_id = symbol_table.get_identifier_id(
                        &pattern.event.to_string(),
                        false,
                        Location::default(),
                        errors,
                    )?;

                    // transform inner_pattern into HIR
                    pattern.pattern.store(true, symbol_table, errors)?;
                    let inner_pattern = pattern.pattern.hir_from_ast(symbol_table, errors)?;
                    let event_pattern =
                        hir::pattern::init(hir::pattern::Kind::present(event_id, inner_pattern));

                    // put event in tuple
                    let idx = events_indices[&event_id];
                    *tuple.get_mut(idx).unwrap() = event_pattern;

                    Ok(None)
                }
                EventPattern::RisingEdge(expr) => {
                    let guard = ast::stream::Expr::binop(ast::expr::Binop::new(
                        operator::BinaryOperator::And,
                        expr.clone(),
                        ast::stream::Expr::unop(ast::expr::Unop::new(
                            operator::UnaryOperator::Not,
                            ast::stream::Expr::fby(ast::stream::Fby::new(
                                ast::stream::Expr::cst(Constant::Boolean(syn::LitBool::new(
                                    false,
                                    macro2::Span::call_site(),
                                ))),
                                expr,
                            )),
                        )),
                    ));
                    Ok(Some(guard))
                }
            }
        }
    }
}
