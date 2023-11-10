use crate::{
    frontend::lir_from_mir::item::{
        array_alias::lir_from_mir as array_alias_lir_from_mir,
        enumeration::lir_from_mir as enumeration_lir_from_mir,
        function::lir_from_mir as function_lir_from_mir,
        node_file::lir_from_mir as node_file_lir_from_mir,
        structure::lir_from_mir as structure_lir_from_mir,
    },
    rust_ast::{file::File, item::Item as LIRItem, project::Project as LIRProject},
    mir::{item::Item, project::Project},
};

/// Transform MIR item into LIR item.
pub fn lir_from_mir(project: Project) -> LIRProject {
    let mut function_file = File::new(format!("functions.rs"));
    let mut typedefs_file = File::new(format!("typedefs.rs"));
    let mut lir_project = LIRProject::new();
    project.items.into_iter().for_each(|item| match item {
        Item::NodeFile(node_file) => {
            let lir_node_file = node_file_lir_from_mir(node_file);
            lir_project.add_file(lir_node_file)
        }
        Item::Function(function) => {
            let lir_function = function_lir_from_mir(function);
            function_file.add_item(LIRItem::Function(lir_function))
        }
        Item::Enumeration(enumeration) => {
            let lir_enumeration = enumeration_lir_from_mir(enumeration);
            typedefs_file.add_item(LIRItem::Enumeration(lir_enumeration))
        }
        Item::Structure(structure) => {
            let lir_structure = structure_lir_from_mir(structure);
            typedefs_file.add_item(LIRItem::Structure(lir_structure))
        }
        Item::ArrayAlias(array_alias) => {
            let lir_array_alias = array_alias_lir_from_mir(array_alias);
            typedefs_file.add_item(LIRItem::TypeAlias(lir_array_alias))
        }
    });
    lir_project.add_file(function_file);
    lir_project.add_file(typedefs_file);

    lir_project
}
