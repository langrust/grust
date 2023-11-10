use crate::{
    frontend::rust_ast_from_mir::item::{
        array_alias::rust_ast_from_mir as array_alias_rust_ast_from_mir,
        enumeration::rust_ast_from_mir as enumeration_rust_ast_from_mir,
        function::rust_ast_from_mir as function_rust_ast_from_mir,
        node_file::rust_ast_from_mir as node_file_rust_ast_from_mir,
        structure::rust_ast_from_mir as structure_rust_ast_from_mir,
    },
    rust_ast::{file::File, item::Item as RustASTItem, project::Project as RustASTProject},
    mir::{item::Item, project::Project},
};

/// Transform MIR item into RustAST item.
pub fn rust_ast_from_mir(project: Project) -> RustASTProject {
    let mut function_file = File::new(format!("functions.rs"));
    let mut typedefs_file = File::new(format!("typedefs.rs"));
    let mut lir_project = RustASTProject::new();
    project.items.into_iter().for_each(|item| match item {
        Item::NodeFile(node_file) => {
            let lir_node_file = node_file_rust_ast_from_mir(node_file);
            lir_project.add_file(lir_node_file)
        }
        Item::Function(function) => {
            let lir_function = function_rust_ast_from_mir(function);
            function_file.add_item(RustASTItem::Function(lir_function))
        }
        Item::Enumeration(enumeration) => {
            let lir_enumeration = enumeration_rust_ast_from_mir(enumeration);
            typedefs_file.add_item(RustASTItem::Enumeration(lir_enumeration))
        }
        Item::Structure(structure) => {
            let lir_structure = structure_rust_ast_from_mir(structure);
            typedefs_file.add_item(RustASTItem::Structure(lir_structure))
        }
        Item::ArrayAlias(array_alias) => {
            let lir_array_alias = array_alias_rust_ast_from_mir(array_alias);
            typedefs_file.add_item(RustASTItem::TypeAlias(lir_array_alias))
        }
    });
    lir_project.add_file(function_file);
    lir_project.add_file(typedefs_file);

    lir_project
}
