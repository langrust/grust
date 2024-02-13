use crate::ast::pattern::Pattern;
use crate::common::scope::Scope;
use crate::error::{Error, TerminationError};
use crate::hir::pattern::Pattern as HIRPattern;
use crate::symbol_table::SymbolTable;

/// Transform AST pattern into HIR pattern.
pub fn hir_from_ast(
    pattern: Pattern,
    stream: bool,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRPattern, TerminationError> {
    match pattern {
        Pattern::Constant { constant, location } => Ok(HIRPattern::Constant { constant, location }),
        Pattern::Identifier { name, location } => {
            let id = if stream {
                symbol_table.insert_signal(name, Scope::Local, true, location.clone(), errors)?
            } else {
                symbol_table.insert_identifier(name, true, location.clone(), errors)?
            };
            Ok(HIRPattern::Identifier { id, location })
        }
        Pattern::Structure {
            name,
            fields,
            location,
        } => Ok(HIRPattern::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(field_name, pattern)| {
                    Ok((
                        field_name,
                        hir_from_ast(pattern, stream, symbol_table, errors)?,
                    ))
                })
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            location,
        }),
        Pattern::Tuple { elements, location } => Ok(HIRPattern::Tuple {
            elements: elements
                .into_iter()
                .map(|pattern| hir_from_ast(pattern, stream, symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            location,
        }),
        Pattern::Some { pattern, location } => Ok(HIRPattern::Some {
            pattern: Box::new(hir_from_ast(*pattern, stream, symbol_table, errors)?),
            location,
        }),
        Pattern::None { location } => Ok(HIRPattern::None { location }),
        Pattern::Default { location } => Ok(HIRPattern::Default { location }),
    }
}
