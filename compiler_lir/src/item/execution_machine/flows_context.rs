prelude! {}

/// A signals context from where components will get their inputs.
#[derive(Debug, PartialEq, Default)]
pub struct FlowsContext {
    pub elements: HashMap<String, Typ>,
}

impl FlowsContext {
    pub fn add_element(&mut self, element_name: String, element_type: &Typ) {
        match self.elements.insert(element_name, element_type.clone()) {
            Some(other_ty) => debug_assert!(other_ty.eq(element_type)),
            None => (),
        }
    }
    pub fn contains_element(&self, element_name: &String) -> bool {
        self.elements.contains_key(element_name)
    }

    pub fn to_syn(self) -> impl Iterator<Item = syn::Item> {
        let FlowsContext { elements } = self;

        // construct Context structure type
        let context_struct = {
            let fields = elements.iter().map(|(element_name, _)| -> syn::Field {
                let name = Ident::new(element_name, Span::call_site());
                let struct_name = Ident::new(&to_camel_case(&element_name), Span::call_site());
                parse_quote! { pub #name: #struct_name }
            });
            let name = Ident::new("Context", Span::call_site());
            let attribute: syn::Attribute =
                parse_quote!(#[derive(Clone, Copy, PartialEq, Default)]);
            parse_quote! {
                #attribute
                pub struct #name {
                    #(#fields),*
                }
            }
        };

        // create an 'init' function
        let init_fun: syn::ImplItem = parse_quote! {
            fn init() -> Context {
                Default::default()
            }
        };

        // create a 'reset' function that resets all signals
        let stmts = elements.iter().map(|(element_name, _)| -> syn::Stmt {
            let name = Ident::new(element_name, Span::call_site());
            parse_quote! { self.#name.reset(); }
        });
        let reset_fun: syn::ImplItem = parse_quote! {
            fn reset(&mut self) {
                #(#stmts)*
            }
        };

        // create the 'Context' implementation
        let context_impl: syn::Item = parse_quote! {
            impl Context {
                #init_fun
                #reset_fun
            }
        };

        // for all element, create a structure representing the updated value
        let items = elements.into_iter().flat_map(|(element_name, element_ty)| {
            let struct_name = Ident::new(&to_camel_case(&element_name), Span::call_site());
            let name = Ident::new(&element_name, Span::call_site());
            let ty = element_ty.to_syn();
            let attribute: syn::Attribute =
                parse_quote!(#[derive(Clone, Copy, PartialEq, Default)]);
            let item_struct: syn::ItemStruct = parse_quote! {
                #attribute
                pub struct #struct_name(#ty, bool);
            };

            let item_impl: syn::ItemImpl = {
                let set_impl: syn::ImplItem = parse_quote! {
                    fn set(&mut self, #name: #ty) {
                        self.0 = #name;
                        self.1 = true;
                    }
                };
                let get_impl: syn::ImplItem = parse_quote! {
                    fn get(&self) -> #ty {
                        self.0
                    }
                };
                let is_new_impl: syn::ImplItem = parse_quote! {
                    fn is_new(&self) -> bool {
                        self.1
                    }
                };
                let reset_impl: syn::ImplItem = parse_quote! {
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

            [syn::Item::Struct(item_struct), syn::Item::Impl(item_impl)]
        });

        items.chain([context_struct, context_impl])
    }
}
