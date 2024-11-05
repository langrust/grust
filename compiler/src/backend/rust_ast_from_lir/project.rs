use std::collections::BTreeSet;

prelude! {
    backend::rust_ast_from_lir::item::{
        array_alias::rust_ast_from_lir as array_alias_rust_ast_from_lir,
        import::rust_ast_from_lir as import_rust_ast_from_lir,
        enumeration::rust_ast_from_lir as enumeration_rust_ast_from_lir,
        execution_machine::rust_ast_from_lir as execution_machine_rust_ast_from_lir,
        function::rust_ast_from_lir as function_rust_ast_from_lir,
        state_machine::rust_ast_from_lir as state_machine_rust_ast_from_lir,
        structure::rust_ast_from_lir as structure_rust_ast_from_lir,
    },
    lir::{Item, Project},
}

/// Transform LIR item into RustAST item.
pub fn rust_ast_from_lir(project: Project) -> Vec<syn::Item> {
    let mut crates = BTreeSet::new();
    let mut rust_items = vec![];

    if conf::greusot() {
        rust_items.push(syn::Item::Use(parse_quote!(
            use creusot_contracts::{ensures, logic, open, prelude, requires, DeepModel};
        )))
    }

    project.items.into_iter().for_each(|item| match item {
        Item::ExecutionMachine(execution_machine) => {
            if conf::test() || conf::demo() {
                let item = execution_machine_rust_ast_from_lir(execution_machine);
                rust_items.push(item);
            }
        }
        Item::StateMachine(state_machine) => {
            let items = state_machine_rust_ast_from_lir(state_machine, &mut crates);
            rust_items.extend(items);
        }
        Item::Function(function) => {
            let item_function = function_rust_ast_from_lir(function, &mut crates);
            rust_items.push(item_function);
        }
        Item::Enumeration(enumeration) => {
            let rust_ast_enumeration = enumeration_rust_ast_from_lir(enumeration);
            rust_items.push(syn::Item::Enum(rust_ast_enumeration))
        }
        Item::Structure(structure) => {
            let rust_ast_structure = structure_rust_ast_from_lir(structure);
            rust_items.push(syn::Item::Struct(rust_ast_structure))
        }
        Item::ArrayAlias(array_alias) => {
            let rust_ast_array_alias = array_alias_rust_ast_from_lir(array_alias);
            rust_items.push(syn::Item::Type(rust_ast_array_alias))
        }
        Item::Import(import) => {
            let rust_ast_import = import_rust_ast_from_lir(import);
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
