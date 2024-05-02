use itertools::Itertools;

use crate::{
    common::r#type::Type, hir::identifier_creator::IdentifierCreator, lir::item::import::Import,
    symbol_table::SymbolTable,
};

impl Type {
    /// Get imports from type.
    pub fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match self {
            Type::Any | Type::Integer | Type::Float | Type::Boolean | Type::Unit => {
                vec![]
            }
            Type::Enumeration { name, .. } => vec![Import::Enumeration(name.clone())],
            Type::Structure { name, .. } => vec![Import::Structure(name.clone())],
            Type::Array(typing, _)
            | Type::Option(typing)
            | Type::Signal(typing)
            | Type::Event(typing) => typing.get_imports(symbol_table),
            Type::Tuple(elements_types) => elements_types
                .iter()
                .flat_map(|typing| typing.get_imports(symbol_table))
                .unique()
                .collect(),
            Type::Abstract(inputs_types, output_type) => {
                let mut imports = output_type.get_imports(symbol_table);
                let mut inputs_imports = inputs_types
                    .iter()
                    .flat_map(|typing| typing.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.append(&mut inputs_imports);
                imports
            }
            Type::NotDefinedYet(_) | Type::Polymorphism(_) | Type::Generic(_) => unreachable!(),
        }
    }

    /// Get generics from type.
    pub fn get_generics(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
    ) -> Vec<(String, Type)> {
        match self {
            Type::Integer
            | Type::Float
            | Type::Boolean
            | Type::Enumeration { .. }
            | Type::Structure { .. }
            | Type::Any
            | Type::Unit => vec![],
            Type::Array(typing, _)
            | Type::Option(typing)
            | Type::Signal(typing)
            | Type::Event(typing) => typing.get_generics(identifier_creator),
            Type::Abstract(inputs_types, output_type) => {
                let mut generics = output_type.get_generics(identifier_creator);
                let mut inputs_generics = inputs_types
                    .iter_mut()
                    .flat_map(|typing| typing.get_generics(identifier_creator))
                    .collect::<Vec<_>>();
                generics.append(&mut inputs_generics);

                // create fresh identifier for the generic type implementing function and add it to the generics
                let fresh_name = identifier_creator.new_type_identifier(String::from("F"));
                generics.push((fresh_name.clone(), self.clone()));

                // modifiy self to be a generic type
                *self = Type::Generic(fresh_name);

                generics
            }
            Type::Tuple(elements_types) => elements_types
                .iter_mut()
                .flat_map(|typing| typing.get_generics(identifier_creator))
                .collect(),
            Type::NotDefinedYet(_) | Type::Polymorphism(_) | Type::Generic(_) => unreachable!(),
        }
    }
}
