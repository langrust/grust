//! Contexts.

prelude! {}

pub struct Simple<'a> {
    pub symbols: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
pub struct Loc<'a> {
    pub loc: &'a Location,
    pub symbols: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
pub struct PatLoc<'a> {
    pub pat: Option<&'a ast::stmt::Pattern>,
    pub loc: &'a Location,
    pub symbols: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
impl<'a> Simple<'a> {
    pub fn new(symbols: &'a mut SymbolTable, errors: &'a mut Vec<Error>) -> Self {
        Self { symbols, errors }
    }
    pub fn add_loc<'b>(&'b mut self, loc: &'b Location) -> Loc<'b> {
        Loc::new(loc, self.symbols, self.errors)
    }
    pub fn add_pat_loc<'b>(
        &'b mut self,
        pat: Option<&'b ast::stmt::Pattern>,
        loc: &'b Location,
    ) -> PatLoc<'b> {
        PatLoc::new(pat, loc, self.symbols, self.errors)
    }
}
impl<'a> Loc<'a> {
    pub fn new(
        loc: &'a Location,
        symbols: &'a mut SymbolTable,
        errors: &'a mut Vec<Error>,
    ) -> Self {
        Self {
            loc,
            symbols,
            errors,
        }
    }
    pub fn add_pat<'b>(&'b mut self, pat: Option<&'b ast::stmt::Pattern>) -> PatLoc<'b> {
        PatLoc::new(pat, self.loc, self.symbols, self.errors)
    }
}
impl<'a> PatLoc<'a> {
    pub fn new(
        pat: Option<&'a ast::stmt::Pattern>,
        loc: &'a Location,
        symbols: &'a mut SymbolTable,
        errors: &'a mut Vec<Error>,
    ) -> Self {
        Self {
            pat,
            loc,
            symbols,
            errors,
        }
    }
    pub fn remove_pat<'b>(&'b mut self) -> Loc<'b> {
        Loc::new(self.loc, self.symbols, self.errors)
    }
    pub fn set_pat(
        &mut self,
        pat: Option<&'a ast::stmt::Pattern>,
    ) -> Option<&'a ast::stmt::Pattern> {
        std::mem::replace(&mut self.pat, pat)
    }
}

/// A signals context from where components will get their inputs.
#[derive(Debug, PartialEq, Default)]
pub struct Flows {
    pub elements: HashMap<String, Typ>,
}

impl Flows {
    pub fn add_element(&mut self, element_name: String, element_type: &Typ) {
        match self.elements.insert(element_name, element_type.clone()) {
            Some(other_ty) => debug_assert!(other_ty.eq(element_type)),
            None => (),
        }
    }
    pub fn contains_element(&self, element_name: &String) -> bool {
        self.elements.contains_key(element_name)
    }

    pub fn into_syn(self) -> impl Iterator<Item = syn::Item> {
        // construct Context structure type
        let context_struct = {
            let fields = self.elements.iter().map(|(element_name, _)| -> syn::Field {
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
        let stmts = self.elements.iter().map(|(element_name, _)| -> syn::Stmt {
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
        let items = self
            .elements
            .into_iter()
            .flat_map(|(element_name, element_ty)| {
                let struct_name = Ident::new(&to_camel_case(&element_name), Span::call_site());
                let name = Ident::new(&element_name, Span::call_site());
                let ty = element_ty.into_syn();
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

pub struct Full<'a, Event> {
    pub imports: &'a mut HashMap<usize, interface::FlowImport>,
    pub exports: &'a HashMap<usize, interface::FlowExport>,
    pub timings: &'a mut Vec<Event>,
    pub symbols: &'a mut SymbolTable,
}

mk_new! { impl{'a, Event} Full<'a, Event> => new {
    imports: &'a mut HashMap<usize, interface::FlowImport>,
    exports: &'a HashMap<usize, interface::FlowExport>,
    timings: &'a mut Vec<Event>,
    symbols: &'a mut SymbolTable,
} }
