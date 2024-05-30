prelude! {
    ast::pattern::{Enumeration, PatSome, Pattern, Structure, Tuple, Typed},
    common::location::Location,
    error::{Error, TerminationError},
    hir::pattern::{Pattern as HIRPattern, PatternKind},
    symbol_table::SymbolTable,
}

use super::HIRFromAST;

impl Typed {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let Typed {
            pattern, typing, ..
        } = self;
        let location = Location::default();

        let pattern = Box::new(pattern.hir_from_ast(symbol_table, errors)?);
        let typing = typing.hir_from_ast(&location, symbol_table, errors)?;
        Ok(PatternKind::Typed { pattern, typing })
    }
}

impl Structure {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let Structure { name, fields, rest } = self;
        let location = Location::default();

        let id = symbol_table.get_struct_id(&name, false, location.clone(), errors)?;
        let mut field_ids = symbol_table
            .get_struct_fields(id)
            .clone()
            .into_iter()
            .map(|id| (symbol_table.get_name(id).clone(), id))
            .collect::<HashMap<_, _>>();

        let fields = fields
            .into_iter()
            .map(|(field_name, optional_pattern)| {
                let id = field_ids.remove(&field_name).map_or_else(
                    || {
                        let error = Error::UnknownField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    },
                    |id| Ok(id),
                )?;
                let pattern = optional_pattern
                    .map(|pattern| pattern.hir_from_ast(symbol_table, errors))
                    .transpose()?;
                Ok((id, pattern))
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        if rest.is_none() {
            // check if there are no missing fields
            field_ids
                .keys()
                .map(|field_name| {
                    let error = Error::MissingField {
                        structure_name: name.clone(),
                        field_name: field_name.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    Err::<(), TerminationError>(TerminationError)
                })
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
        }

        Ok(PatternKind::Structure { id, fields })
    }
}

impl Enumeration {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let Enumeration {
            enum_name,
            elem_name,
        } = self;
        let location = Location::default();

        let enum_id = symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
        let elem_id = symbol_table.get_enum_elem_id(
            &elem_name,
            &enum_name,
            false,
            location.clone(),
            errors,
        )?;
        Ok(PatternKind::Enumeration { enum_id, elem_id })
    }
}

impl Tuple {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let Tuple { elements } = self;
        Ok(PatternKind::Tuple {
            elements: elements
                .into_iter()
                .map(|pattern| pattern.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl PatSome {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let PatSome { pattern } = self;
        Ok(PatternKind::Some {
            pattern: Box::new(pattern.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl HIRFromAST for Pattern {
    type HIR = HIRPattern;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let location = Location::default();

        let kind = match self {
            Pattern::Constant(constant) => PatternKind::Constant { constant },
            Pattern::Identifier(name) => {
                let id = symbol_table.get_identifier_id(&name, false, location.clone(), errors)?;
                PatternKind::Identifier { id }
            }
            Pattern::Typed(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::Structure(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::Enumeration(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::Tuple(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            // Pattern::None => PatternKind::None,
            Pattern::Default => PatternKind::Default,
        };

        Ok(HIRPattern {
            kind,
            typing: None,
            location,
        })
    }
}

impl Pattern {
    pub fn store(
        &self,
        is_declaration: bool,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, TerminationError> {
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
                    let id =
                        symbol_table.get_identifier_id(name, false, location.clone(), errors)?;
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
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
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
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect()),
            Pattern::Constant(_) | Pattern::Enumeration(_) | Pattern::Default => Ok(vec![]),
        }
    }

    pub fn get_simple_patterns(self) -> Vec<Pattern> {
        match self {
            Pattern::Identifier(_) | Pattern::Typed(_) => vec![self],
            Pattern::Tuple(Tuple { elements }) => elements
                .into_iter()
                .flat_map(|pattern| pattern.get_simple_patterns())
                .collect(),
            _ => todo!(),
        }
    }
}
