//! [Project] module.

prelude! {}

/// A project structure.
pub struct Project {
    /// The project's items.
    pub items: Vec<Item>,
}

impl Project {
    /// Transform [ir2] item into RustAST item.
    pub fn into_syn(self) -> Vec<syn::Item> {
        let mut crates = BTreeSet::new();
        let mut rust_items = vec![];

        if conf::greusot() {
            rust_items.push(syn::Item::Use(parse_quote!(
                use creusot_contracts::{ensures, logic, open, prelude, requires, DeepModel};
            )))
        }

        self.items.into_iter().for_each(|item| match item {
            Item::ExecutionMachine(execution_machine) => {
                if conf::test() || conf::demo() {
                    let item = execution_machine.into_syn();
                    rust_items.push(item);
                }
            }
            Item::StateMachine(state_machine) => {
                let items = state_machine.into_syn(&mut crates);
                rust_items.extend(items);
            }
            Item::Function(function) => {
                let item_function = function.into_syn(&mut crates);
                rust_items.push(item_function);
            }
            Item::Enumeration(enumeration) => {
                let rust_ast_enumeration = enumeration.into_syn();
                rust_items.push(syn::Item::Enum(rust_ast_enumeration))
            }
            Item::Structure(structure) => {
                let rust_ast_structure = structure.into_syn();
                rust_items.push(syn::Item::Struct(rust_ast_structure))
            }
            Item::ArrayAlias(array_alias) => {
                let rust_ast_array_alias = array_alias.into_syn();
                rust_items.push(syn::Item::Type(rust_ast_array_alias))
            }
            Item::Import(import) => {
                let rust_ast_import = import.into_syn();
                rust_items.push(syn::Item::Use(rust_ast_import))
            }
        });

        // remove duplicated imports
        use itertools::Itertools; // for `unique`
        rust_items = std::mem::take(&mut rust_items)
            .into_iter()
            .unique()
            .collect::<Vec<_>>();

        rust_items
    }
}
