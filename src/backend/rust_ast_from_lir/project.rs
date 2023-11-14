use std::path::Path;

use crate::{
    backend::rust_ast_from_lir::item::{
        array_alias::rust_ast_from_lir as array_alias_rust_ast_from_lir,
        enumeration::rust_ast_from_lir as enumeration_rust_ast_from_lir,
        function::rust_ast_from_lir as function_rust_ast_from_lir,
        node_file::rust_ast_from_lir as node_file_rust_ast_from_lir,
        structure::rust_ast_from_lir as structure_rust_ast_from_lir,
    },
    lir::{item::Item, project::Project},
    rust_ast::{
        file::File,
        item::{import::Import, Item as RustASTItem},
        project::Project as RustASTProject,
    },
};

/// Transform LIR item into RustAST item.
pub fn rust_ast_from_lir(project: Project) -> RustASTProject {
    let mut rust_ast_project = RustASTProject::new();

    let mut function_file = File::new(format!("src/functions.rs"));
    let mut typedefs_file = File::new(format!("src/typedefs.rs"));

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

    let mut lib_file = File::new(format!("src/lib.rs"));
    rust_ast_project.files.iter().for_each(|file| {
        let module_name = Path::new(&file.path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let module_import = Import::Module {
            public_visibility: true,
            name: module_name,
        };
        lib_file.add_item(RustASTItem::Import(module_import))
    });
    rust_ast_project.add_file(lib_file);

    rust_ast_project
}
