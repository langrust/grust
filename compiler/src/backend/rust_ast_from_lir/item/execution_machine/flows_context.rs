use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::common::convert_case::camel_case;
use crate::lir::item::execution_machine::flows_context::FlowsContext;
use proc_macro2::Span;
use quote::format_ident;
use syn::*;

/// Transform LIR flows context into a 'Context' structure
/// that implements some useful functions.
pub fn rust_ast_from_lir(flows_context: FlowsContext) -> Vec<Item> {
    let FlowsContext {
        elements,
        components,
    } = flows_context;

    // construct Context structure type
    let context_struct = {
        let fields = elements.iter().map(|(element_name, element_ty)| {
            let name = Ident::new(element_name, Span::call_site());
            let ty = type_rust_ast_from_lir(element_ty.clone());
            Field {
                attrs: vec![],
                vis: Visibility::Public(Default::default()),
                ident: Some(name),
                colon_token: Default::default(),
                ty,
                mutability: FieldMutability::None,
            }
        });
        let name = Ident::new("Context", Span::call_site());
        parse_quote! {
            #[derive(Clone, Copy, Debug, PartialEq, Default)]
            pub struct #name {
                #(#fields),*
            }
        }
    };

    // create an 'init' function
    let mut impl_items: Vec<ImplItem> = vec![parse_quote! {
        fn init() -> Context {
            Default::default()
        }
    }];

    // for all components, create its input generator
    components
        .into_iter()
        .for_each(|(component_name, input_fields)| {
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            let component_input_name =
                format_ident!("{}", camel_case(&format!("{component_name}Input")));
            let component_event_name =
                format_ident!("{}", camel_case(&format!("{component_name}Event")));

            let input_fields: Vec<FieldValue> = input_fields
                .into_iter()
                .map(|(field_name, in_context)| {
                    let field_id = Ident::new(&field_name, Span::call_site());
                    let in_context_id = Ident::new(&in_context, Span::call_site());
                    let expr: Expr = parse_quote!(self.#in_context_id);
                    parse_quote! { #field_id : #expr }
                })
                .collect();

            let function: ImplItem = parse_quote! {
                fn #input_getter(&self, event: #component_event_name) -> #component_input_name {
                    #component_input_name { #(#input_fields),* }
                }
            };

            impl_items.push(function)
        });

    // create the 'Context' implementation
    let context_impl: Item = parse_quote! {
        impl Context {
            #(#impl_items)*
        }
    };
    vec![context_struct, context_impl]
}
