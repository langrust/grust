use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::pattern::{Pattern, PatternKind};
use crate::symbol_table::SymbolTable;

impl Pattern {
    /// Tries to type the given construct.
    pub fn typing(
        &mut self,
        expected_type: &Type,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            PatternKind::Constant { ref constant } => {
                let pattern_type = constant.get_type();
                pattern_type.eq_check(&expected_type, self.location.clone(), errors)?;
                self.typing = Some(pattern_type);
                Ok(())
            }
            PatternKind::Identifier { id } => {
                symbol_table.set_type(id, expected_type.clone());
                self.typing = Some(expected_type.clone());
                Ok(())
            }
            PatternKind::Typed {
                ref mut pattern,
                ref typing,
            } => {
                typing.eq_check(&expected_type, self.location.clone(), errors)?;
                pattern.typing(expected_type, symbol_table, errors)
            }
            PatternKind::Structure {
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
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;
                self.typing = Some(Type::Structure {
                    name: symbol_table.get_name(*id).clone(),
                    id: *id,
                });
                Ok(())
            }
            PatternKind::Enumeration { ref enum_id, .. } => {
                self.typing = Some(Type::Enumeration {
                    name: symbol_table.get_name(*enum_id).clone(),
                    id: *enum_id,
                });
                Ok(())
            }
            PatternKind::Tuple { ref mut elements } => match expected_type {
                Type::Tuple(types) => {
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
                        .collect::<Vec<Result<(), TerminationError>>>()
                        .into_iter()
                        .collect::<Result<(), TerminationError>>()?;
                    let types = elements
                        .iter()
                        .map(|pattern| pattern.get_type().unwrap().clone())
                        .collect();
                    self.typing = Some(Type::Tuple(types));
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
            PatternKind::Some { ref mut pattern } => match expected_type {
                Type::Option(expected_type) => {
                    pattern.typing(expected_type, symbol_table, errors)?;
                    let pattern_type = pattern.get_type().unwrap().clone();
                    self.typing = Some(Type::Option(Box::new(pattern_type)));
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
            PatternKind::None => {
                self.typing = Some(Type::Option(Box::new(Type::Any)));
                Ok(())
            }
            PatternKind::Default => {
                self.typing = Some(Type::Any);
                Ok(())
            }
        }
    }

    /// Tries to construct the type of the given construct.
    pub fn construct_statement_type(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self.kind {
            PatternKind::Constant { .. }
            | PatternKind::Structure { .. }
            | PatternKind::Enumeration { .. }
            | PatternKind::Some { .. }
            | PatternKind::None
            | PatternKind::Default => {
                let error = Error::NotStatementPattern {
                    location: self.location.clone(),
                };
                errors.push(error);
                return Err(TerminationError);
            }
            PatternKind::Identifier { id } => {
                let typing = symbol_table.get_type(id);
                self.typing = Some(typing.clone());
                Ok(typing.clone())
            }
            PatternKind::Typed {
                ref mut pattern,
                ref typing,
            } => {
                pattern.typing(typing, symbol_table, errors)?;
                self.typing = Some(typing.clone());
                Ok(typing.clone())
            }
            PatternKind::Tuple { ref mut elements } => {
                let types = elements
                    .iter_mut()
                    .map(|pattern| pattern.construct_statement_type(symbol_table, errors))
                    .collect::<Vec<Result<_, TerminationError>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, TerminationError>>()?;

                self.typing = Some(Type::Tuple(types));
                Ok(Type::Tuple(types))
            }
        }
    }
}
