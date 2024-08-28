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
pub fn rust_ast_from_lir(flows_context: FlowsContext) -> impl Iterator<Item = Item> {
    let FlowsContext {
        elements,
        components,
    } = flows_context;

    // construct Context structure type
    let context_struct = {
        let fields = elements.iter().map(|(element_name, _)| -> Field {
            let name = Ident::new(element_name, Span::call_site());
            let struct_name = Ident::new(&to_camel_case(&element_name), Span::call_site());
            parse_quote! { pub #name: #struct_name }
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

    // create a 'reset' function that resets all signals
    let stmts = elements.iter().map(|(element_name, _)| -> Stmt {
        let name = Ident::new(element_name, Span::call_site());
        parse_quote! { self.#name.reset(); }
    });
    let reset_fun = parse_quote! {
        fn reset(&mut self) {
            #(#stmts)*
        }
    };
    impl_items.push(reset_fun);

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
                    let expr: Expr = parse_quote!(self.#in_context_id.get());
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

    // for all element, create a structure representing the updated value
    let items = elements.into_iter().flat_map(|(element_name, element_ty)| {
        let struct_name = Ident::new(&to_camel_case(&element_name), Span::call_site());
        let name = Ident::new(&element_name, Span::call_site());
        let ty = type_rust_ast_from_lir(element_ty.clone());
        let attribute: Attribute = parse_quote!(#[derive(Clone, Copy, PartialEq, Default)]);
        let item_struct: ItemStruct = parse_quote! {
            #attribute
            pub struct #struct_name(#ty, bool);
        };

        let item_impl: ItemImpl = {
            let set_impl: ImplItem = parse_quote! {
                fn set(&mut self, #name: #ty) {
                    self.0 = #name;
                    self.1 = true;
                }
            };
            let get_impl: ImplItem = parse_quote! {
                fn get(&self) -> #ty {
                    self.0
                }
            };
            let is_new_impl: ImplItem = parse_quote! {
                fn is_new(&self) -> bool {
                    self.1
                }
            };
            let reset_impl: ImplItem = parse_quote! {
                fn reset(&mut self) {
                    self.1 = false;
                }
            };
            parse_quote! {
                impl #struct_name {
                    #set_impl
                    #get_impl
                    #is_new_impl
                    #reset_impl
                }
            }
        };

        [Item::Struct(item_struct), Item::Impl(item_impl)]
    });

    items.chain([context_struct, context_impl])
}
