use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

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
use proc_macro2::Span;
use syn::*;

#[derive(Debug)]
/// Rust project resulting from compiling.
pub struct RustASTProject {
    /// Project's directory.
    pub directory: String,
    files: BTreeMap<String, File>,
    crates: BTreeSet<String>,
}
impl RustASTProject {
    fn new() -> Self {
        RustASTProject {
            files: Default::default(),
            directory: Default::default(),
            crates: Default::default(),
        }
    }

    fn add_file(&mut self, path: &str, file: File) {
        self.files.insert(path.to_owned(), file);
    }

    /// Set project's directory.
    pub fn set_parent<P>(&mut self, path: P)
    where
        P: AsRef<std::path::Path>,
    {
        let subdirectory = std::mem::take(&mut self.directory);
        self.directory = path
            .as_ref()
            .join(subdirectory)
            .into_os_string()
            .into_string()
            .unwrap();
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
            let (path, rust_ast_node_file) =
                node_file_rust_ast_from_lir(node_file, &mut rust_ast_project.crates);
            rust_ast_project.add_file(&path, rust_ast_node_file)
        }
        Item::Function(function) => {
            let mut rust_ast_function =
                function_rust_ast_from_lir(function, &mut rust_ast_project.crates);
            function_file.items.append(&mut rust_ast_function);
            // remove duplicated imports between functions
            function_file.items = std::mem::take(&mut function_file.items)
                .into_iter()
                .unique()
                .collect::<Vec<_>>();
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
    rust_ast_project.files.iter().for_each(|(path, _)| {
        let module_name = Path::new(&path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let module_ident = Ident::new(&module_name, Span::call_site());
        let module_decl = parse_quote! { pub mod #module_ident; };
        lib_file.items.push(syn::Item::Mod(module_decl))
    });
    rust_ast_project.add_file("src/lib.rs", lib_file);

    rust_ast_project
}
