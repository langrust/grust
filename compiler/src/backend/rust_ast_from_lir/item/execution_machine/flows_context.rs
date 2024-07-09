prelude! {
    macro2::Span,
    quote::format_ident,
    syn::*,
}

prelude! { just
    backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    lir::item::execution_machine::flows_context::FlowsContext,
}

/// Transform LIR flows context into a 'Context' structure
/// that implements some useful functions.
pub fn rust_ast_from_lir(flows_context: FlowsContext) -> Vec<Item> {
    if conf::greusot() {
        return vec![];
    }

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
        let attribute: Attribute = parse_quote!(#[derive(Clone, Copy, PartialEq, Default)]);
        parse_quote! {
            #attribute
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
        .for_each(|(component_name, (events_fields, signals_fields))| {
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            let component_input_name =
                format_ident!("{}", to_camel_case(&format!("{component_name}Input")));

            let mut input_fields: Vec<FieldValue> = signals_fields
                .into_iter()
                .map(|(field_name, in_context)| {
                    let field_id = Ident::new(&field_name, Span::call_site());
                    let in_context_id = Ident::new(&in_context, Span::call_site());
                    let expr: Expr = parse_quote!(self.#in_context_id);
                    parse_quote! { #field_id : #expr }
                })
                .collect();

            let args: Vec<FnArg> = events_fields
                .into_iter()
                .map(|(field_name, event_name, event_ty)| {
                    // add input field
                    let field_id = Ident::new(&field_name, Span::call_site());
                    let event_id = Ident::new(&event_name, Span::call_site());
                    input_fields.push(parse_quote! { #field_id : #event_id });

                    let event_ty = type_rust_ast_from_lir(event_ty.convert());
                    parse_quote! { #event_id: #event_ty }
                })
                .collect();

            let function: ImplItem = parse_quote! {
                fn #input_getter(&self, #(#args),*) -> #component_input_name {
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
