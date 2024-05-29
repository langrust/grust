use crate::common::location::Location;
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Type {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        // precondition: Typedefs are stored in symbol table
        // postcondition: construct a new Type without `Type::NotDefinedYet`
        match self {
            Type::Array(array_type, array_size) => Ok(Type::Array(
                Box::new(array_type.hir_from_ast(location, symbol_table, errors)?),
                array_size,
            )),
            Type::SMEvent(event_type) => Ok(Type::SMEvent(Box::new(event_type.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?))),
            Type::SMTimeout(timeout_type) => Ok(Type::SMTimeout(Box::new(timeout_type.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?))),
            Type::Tuple(tuple_types) => Ok(Type::Tuple(
                tuple_types
                    .into_iter()
                    .map(|element_type| element_type.hir_from_ast(location, symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Type::NotDefinedYet(name) => symbol_table
                .get_struct_id(&name, false, location.clone(), &mut vec![])
                .map(|id| Type::Structure {
                    name: name.clone(),
                    id,
                })
                .or_else(|_| {
                    symbol_table
                        .get_enum_id(&name, false, location.clone(), &mut vec![])
                        .map(|id| Type::Enumeration {
                            name: name.clone(),
                            id,
                        })
                })
                .or_else(|_| {
                    let id = symbol_table.get_array_id(&name, false, location.clone(), errors)?;
                    Ok(symbol_table.get_array(id))
                }),
            Type::Abstract(inputs_types, output_type) => {
                let inputs_types = inputs_types
                    .into_iter()
                    .map(|input_type| input_type.hir_from_ast(location, symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;
                let output_type = output_type.hir_from_ast(location, symbol_table, errors)?;
                Ok(Type::Abstract(inputs_types, Box::new(output_type)))
            }
            Type::Signal(signal_type) => Ok(Type::Signal(Box::new(signal_type.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?))),
            Type::Event(event_type) => Ok(Type::Event(Box::new(event_type.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?))),
            Type::Timeout(timeout_type) => Ok(Type::Timeout(Box::new(timeout_type.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?))),
            Type::Integer | Type::Float | Type::Boolean | Type::Unit| Type::Time => Ok(self),
            Type::Enumeration { .. }    // no enumeration at this time: they are `NotDefinedYet`
            | Type::Structure { .. }    // no structure at this time: they are `NotDefinedYet`
            | Type::ComponentEvent      // users can not write `ComponentEvent` type
            | Type::Any                 // users can not write `Any` type
            | Type::Polymorphism(_)     // users can not write `Polymorphism` type
            | Type::Generic(_)          // users can not write `Generic` type
             => unreachable!(),
        }
    }
}
