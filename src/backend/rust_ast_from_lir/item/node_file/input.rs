use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::common::convert_case::camel_case;
use crate::lir::item::node_file::input::{Input, InputElement};
use quote::format_ident;
use syn::*;

/// Transform LIR input into RustAST structure.
pub fn rust_ast_from_lir(input: Input) -> ItemStruct {
    let mut fields: Vec<Field> = Vec::new();
    for InputElement { identifier, r#type } in input.elements {
        let ty = type_rust_ast_from_lir(r#type);
        let identifier = format_ident!("{identifier}");
        fields.push(parse_quote! { pub #identifier : #ty });
    }

    let name = format_ident!("{}", camel_case(&format!("{}Input", input.node_name)));
    parse_quote! {
        pub struct #name {
            #(#fields),*
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::node_file::input::rust_ast_from_lir;
    use crate::common::r#type::Type;
    use crate::lir::item::node_file::input::{Input, InputElement};
    use syn::*;

    #[test]
    fn should_create_rust_ast_structure_from_lir_node_input() {
        let input = Input {
            node_name: format!("Node"),
            elements: vec![InputElement {
                identifier: format!("i"),
                r#type: Type::Integer,
            }],
        };
        let control = parse_quote!(
            pub struct NodeInput {
                pub i: i64
            }
        );

        assert_eq!(rust_ast_from_lir(input), control)
    }
}
