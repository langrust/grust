use crate::{
    frontend::rust_ast_from_lir::item::{
        array_alias::rust_ast_from_lir as array_alias_rust_ast_from_lir,
        enumeration::rust_ast_from_lir as enumeration_rust_ast_from_lir,
        function::rust_ast_from_lir as function_rust_ast_from_lir,
        node_file::rust_ast_from_lir as node_file_rust_ast_from_lir,
        structure::rust_ast_from_lir as structure_rust_ast_from_lir,
    },
    rust_ast::{file::File, item::Item as RustASTItem, project::Project as RustASTProject},
    lir::{item::Item, project::Project},
};

/// Transform LIR item into RustAST item.
pub fn rust_ast_from_lir(project: Project) -> RustASTProject {
    let mut function_file = File::new(format!("functions.rs"));
    let mut typedefs_file = File::new(format!("typedefs.rs"));
    let mut rust_ast_project = RustASTProject::new();
    project.items.into_iter().for_each(|item| match item {
        Item::NodeFile(node_file) => {
            let rust_ast_node_file = node_file_rust_ast_from_lir(node_file);
            rust_ast_project.add_file(rust_ast_node_file)
        }
        Item::Function(function) => {
            let rust_ast_function = function_rust_ast_from_lir(function);
            function_file.add_item(RustASTItem::Function(rust_ast_function))
        }
        Item::Enumeration(enumeration) => {
            let rust_ast_enumeration = enumeration_rust_ast_from_lir(enumeration);
            typedefs_file.add_item(RustASTItem::Enumeration(rust_ast_enumeration))
        }
        Item::Structure(structure) => {
            let rust_ast_structure = structure_rust_ast_from_lir(structure);
            typedefs_file.add_item(RustASTItem::Structure(rust_ast_structure))
        }
        Item::ArrayAlias(array_alias) => {
            let rust_ast_array_alias = array_alias_rust_ast_from_lir(array_alias);
            typedefs_file.add_item(RustASTItem::TypeAlias(rust_ast_array_alias))
        }
    });
    rust_ast_project.add_file(function_file);
    rust_ast_project.add_file(typedefs_file);

    rust_ast_project
}
