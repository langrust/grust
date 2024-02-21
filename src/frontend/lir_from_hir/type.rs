use itertools::Itertools;

use crate::{common::r#type::Type, lir::item::import::Import, symbol_table::SymbolTable};

impl Type {
    pub fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match self {
            Type::Any | Type::Integer | Type::Float | Type::Boolean | Type::String | Type::Unit => {
                vec![]
            }
            Type::Enumeration { name, .. } => vec![Import::Enumeration(name.clone())],
            Type::Structure { name, .. } => vec![Import::Structure(name.clone())],
            Type::Array(typing, _) | Type::Option(typing) => typing.get_imports(symbol_table),
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
            Type::NotDefinedYet(_) | Type::Polymorphism(_) => unreachable!(),
        }
    }
}
