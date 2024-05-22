use std::collections::BTreeSet;

use crate::{
    backend::rust_ast_from_lir::item::{
        array_alias::rust_ast_from_lir as array_alias_rust_ast_from_lir,
        enumeration::rust_ast_from_lir as enumeration_rust_ast_from_lir,
        function::rust_ast_from_lir as function_rust_ast_from_lir,
        node_file::rust_ast_from_lir as node_file_rust_ast_from_lir,
        structure::rust_ast_from_lir as structure_rust_ast_from_lir,
    },
    lir::{item::Item, project::Project},
};
use itertools::Itertools;

/// Transform LIR item into RustAST item.
pub fn rust_ast_from_lir(project: Project) -> Vec<syn::Item> {
    let mut crates = BTreeSet::new();
    let mut rust_items = vec![];

    project.items.into_iter().for_each(|item| match item {
        Item::StateMachine(node_file) => {
            let mut items = node_file_rust_ast_from_lir(node_file, &mut crates);
            rust_items.append(&mut items);
        }
        Item::Function(function) => {
            let mut items = function_rust_ast_from_lir(function, &mut crates);
            rust_items.append(&mut items);
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
    });

    // remove duplicated imports
    rust_items = std::mem::take(&mut rust_items)
        .into_iter()
        .unique()
        .collect::<Vec<_>>();

    rust_items
}
