use std::collections::HashMap;

use crate::ast::pattern::{Enumeration, Pattern, Some, Structure, Tuple};
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::pattern::{Pattern as HIRPattern, PatternKind};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl Structure {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let Structure { name, fields } = self;
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
            .map(|(field_name, pattern)| {
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
                let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                Ok((id, pattern))
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

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

impl Some {
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<PatternKind, TerminationError> {
        let Some { pattern } = self;
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
                let id =
                    symbol_table.insert_identifier(name, None, true, location.clone(), errors)?;
                PatternKind::Identifier { id }
            }
            Pattern::Structure(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::Enumeration(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::Tuple(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::Some(pattern) => pattern.hir_from_ast(symbol_table, errors)?,
            Pattern::None => PatternKind::None,
            Pattern::Default => PatternKind::Default,
        };

        Ok(HIRPattern {
            kind,
            typing: None,
            location,
        })
    }
}
