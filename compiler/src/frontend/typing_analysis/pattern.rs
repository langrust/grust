//! LanGRust [Pattern] typing analysis module.

prelude! {
    hir::pattern::{Pattern, Kind},
}

impl Pattern {
    /// Tries to type the given construct.
    pub fn typing(
        &mut self,
        expected_type: &Typ,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        match self.kind {
            Kind::Constant { ref constant } => {
                let pattern_type = constant.get_type();
                pattern_type.eq_check(&expected_type, self.location.clone(), errors)?;
                self.typing = Some(pattern_type);
                Ok(())
            }
            Kind::Identifier { id } => {
                symbol_table.set_type(id, expected_type.clone());
                self.typing = Some(expected_type.clone());
                Ok(())
            }
            Kind::Typed {
                ref mut pattern,
                ref typing,
            } => {
                typing.eq_check(&expected_type, self.location.clone(), errors)?;
                pattern.typing(expected_type, symbol_table, errors)
            }
            Kind::Structure {
                ref id,
                ref mut fields,
            } => {
                fields
                    .iter_mut()
                    .map(|(id, optional_pattern)| {
                        let expected_type = symbol_table.get_type(*id).clone();
                        if let Some(pattern) = optional_pattern {
                            pattern.typing(&expected_type, symbol_table, errors)?;
                            // check pattern type
                            let pattern_type = pattern.get_type().unwrap();
                            pattern_type.eq_check(&expected_type, self.location.clone(), errors)
                        } else {
                            Ok(())
                        }
                    })
                    .collect::<Vec<TRes<()>>>()
                    .into_iter()
                    .collect::<TRes<()>>()?;
                self.typing = Some(Typ::structure(symbol_table.get_name(*id), *id));
                Ok(())
            }
            Kind::Enumeration { ref enum_id, .. } => {
                self.typing = Some(Typ::enumeration(symbol_table.get_name(*enum_id), *enum_id));
                Ok(())
            }
            Kind::Tuple { ref mut elements } => match expected_type {
                Typ::Tuple {
                    elements: types, ..
                } => {
                    if elements.len() != types.len() {
                        let error = Error::IncompatibleTuple {
                            location: self.location.clone(),
                        };
                        errors.push(error);
                        return Err(TerminationError);
                    }
                    elements
                        .iter_mut()
                        .zip(types)
                        .map(|(pattern, expected_type)| {
                            pattern.typing(expected_type, symbol_table, errors)
                        })
                        .collect::<Vec<TRes<()>>>()
                        .into_iter()
                        .collect::<TRes<()>>()?;
                    let types = elements
                        .iter()
                        .map(|pattern| pattern.get_type().unwrap().clone())
                        .collect();
                    self.typing = Some(Typ::tuple(types));
                    Ok(())
                }
                _ => {
                    let error = Error::ExpectTuplePattern {
                        location: self.location.clone(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
            },
            Kind::Some { ref mut pattern } => match expected_type {
                Typ::SMEvent { ty, .. } => {
                    pattern.typing(ty, symbol_table, errors)?;
                    let pattern_type = pattern.get_type().unwrap().clone();
                    self.typing = Some(Typ::sm_event(pattern_type));
                    Ok(())
                }
                _ => {
                    let error = Error::ExpectOptionPattern {
                        location: self.location.clone(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
            },
            Kind::None => {
                self.typing = Some(Typ::sm_event(Typ::Any));
                Ok(())
            }
            Kind::Default => {
                self.typing = Some(Typ::any());
                Ok(())
            }
            Kind::PresentEvent {
                event_id,
                ref mut pattern,
            } => {
                let typing = symbol_table.get_type(event_id).clone();
                expected_type.eq_check(&typing, self.location.clone(), errors)?;

                match &typing {
                    Typ::SMEvent { ty, .. } => pattern.typing(&ty, symbol_table, errors)?,
                    _ => unreachable!(),
                };

                self.typing = Some(typing);
                Ok(())
            }
            Kind::NoEvent { event_id } => {
                let typing = symbol_table.get_type(event_id).clone();
                self.typing = Some(typing);
                Ok(())
            }
        }
    }

    /// Tries to construct the type of the given construct.
    pub fn construct_statement_type(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        match self.kind {
            Kind::Constant { .. }
            | Kind::Structure { .. }
            | Kind::Enumeration { .. }
            | Kind::Some { .. }
            | Kind::NoEvent { .. }
            | Kind::PresentEvent { .. }
            | Kind::None
            | Kind::Default => {
                let error = Error::NotStatementPattern {
                    location: self.location.clone(),
                };
                errors.push(error);
                return Err(TerminationError);
            }
            Kind::Identifier { id } => {
                let typing = symbol_table.get_type(id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            Kind::Typed {
                ref mut pattern,
                ref typing,
            } => {
                pattern.typing(typing, symbol_table, errors)?;
                self.typing = Some(typing.clone());
                Ok(())
            }
            Kind::Tuple { ref mut elements } => {
                let types = elements
                    .iter_mut()
                    .map(|pattern| {
                        pattern.construct_statement_type(symbol_table, errors)?;
                        Ok(pattern.typing.as_ref().unwrap().clone())
                    })
                    .collect::<Vec<TRes<_>>>()
                    .into_iter()
                    .collect::<TRes<Vec<_>>>()?;

                self.typing = Some(Typ::tuple(types));
                Ok(())
            }
        }
    }
}
