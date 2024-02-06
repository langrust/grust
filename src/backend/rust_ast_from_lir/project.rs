use std::{collections::HashMap, path::Path};

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
use proc_macro2::Span;
use syn::*;

struct RustASTProject {
    files: HashMap<String, File>,
}
impl RustASTProject {
    fn new() -> Self {
        RustASTProject {
            files: Default::default(),
        }
    }

    fn add_file(&mut self, path: &str, file: File) {
        self.files.insert(path.to_owned(), file);
    }
}

/// Transform LIR item into RustAST item.
pub fn rust_ast_from_lir(project: Project) -> RustASTProject {
    let mut rust_ast_project = RustASTProject::new();

    let mut function_file = File {
        shebang: None,
        items: Default::default(),
        attrs: vec![],
    };
    let mut typedefs_file = File {
        shebang: None,
        items: Default::default(),
        attrs: vec![],
    };

    project.items.into_iter().for_each(|item| match item {
        Item::NodeFile(node_file) => {
            let (path, rust_ast_node_file) = node_file_rust_ast_from_lir(node_file);
            rust_ast_project.add_file(&path, rust_ast_node_file)
        }
        Item::Function(function) => {
            let rust_ast_function = function_rust_ast_from_lir(function);
            function_file.items.push(syn::Item::Fn(rust_ast_function))
        }
        Item::Enumeration(enumeration) => {
            let rust_ast_enumeration = enumeration_rust_ast_from_lir(enumeration);
            typedefs_file
                .items
                .push(syn::Item::Enum(rust_ast_enumeration))
        }
        Item::Structure(structure) => {
            let rust_ast_structure = structure_rust_ast_from_lir(structure);
            typedefs_file
                .items
                .push(syn::Item::Struct(rust_ast_structure))
        }
        Item::ArrayAlias(array_alias) => {
            let rust_ast_array_alias = array_alias_rust_ast_from_lir(array_alias);
            typedefs_file
                .items
                .push(syn::Item::Type(rust_ast_array_alias))
        }
    });

    rust_ast_project.add_file("src/functions.rs", function_file);
    rust_ast_project.add_file("src/typedefs.rs", typedefs_file);

    let mut lib_file = File {
        shebang: None,
        items: Default::default(),
        attrs: vec![],
    };
    rust_ast_project.files.iter().for_each(|(path, file)| {
        let module_name = Path::new(&path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let module_ident = Ident::new(&module_name, Span::call_site());
        let module_import = parse_quote! { use #module_ident; };
        lib_file.items.push(syn::Item::Use(module_import))
    });
    rust_ast_project.add_file("src/lib.rs", lib_file);

    rust_ast_project
}
