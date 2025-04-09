//! Contexts.

prelude! {}

pub struct Simple<'a> {
    pub ctx0: &'a mut Ctx,
    pub errors: &'a mut Vec<Error>,
}
impl ops::Deref for Simple<'_> {
    type Target = Ctx;
    fn deref(&self) -> &Self::Target {
        self.ctx0
    }
}
impl ops::DerefMut for Simple<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx0
    }
}
pub struct WithLoc<'a> {
    pub loc: Loc,
    pub ctx0: &'a mut Ctx,
    pub errors: &'a mut Vec<Error>,
}
impl ops::Deref for WithLoc<'_> {
    type Target = Ctx;
    fn deref(&self) -> &Self::Target {
        self.ctx0
    }
}
impl ops::DerefMut for WithLoc<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx0
    }
}
pub struct PatLoc<'a> {
    pub pat: Option<&'a ir0::stmt::Pattern>,
    pub loc: Loc,
    pub ctx0: &'a mut Ctx,
    pub errors: &'a mut Vec<Error>,
}
impl ops::Deref for PatLoc<'_> {
    type Target = Ctx;
    fn deref(&self) -> &Self::Target {
        self.ctx0
    }
}
impl ops::DerefMut for PatLoc<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx0
    }
}
impl<'a> Simple<'a> {
    pub fn new(ctx0: &'a mut Ctx, errors: &'a mut Vec<Error>) -> Self {
        Self { ctx0, errors }
    }
    pub fn add_loc<'b>(&'b mut self, loc: Loc) -> WithLoc<'b> {
        WithLoc::new(loc, self.ctx0, self.errors)
    }
    pub fn add_pat_loc<'b>(
        &'b mut self,
        pat: Option<&'b ir0::stmt::Pattern>,
        loc: Loc,
    ) -> PatLoc<'b> {
        PatLoc::new(pat, loc, self.ctx0, self.errors)
    }
}
impl<'a> WithLoc<'a> {
    pub fn new(loc: Loc, ctx0: &'a mut Ctx, errors: &'a mut Vec<Error>) -> Self {
        Self { loc, ctx0, errors }
    }
    pub fn add_pat<'b>(&'b mut self, pat: Option<&'b ir0::stmt::Pattern>) -> PatLoc<'b> {
        PatLoc::new(pat, self.loc, self.ctx0, self.errors)
    }
}
impl<'a> PatLoc<'a> {
    pub fn new(
        pat: Option<&'a ir0::stmt::Pattern>,
        loc: impl Into<Loc>,
        ctx0: &'a mut Ctx,
        errors: &'a mut Vec<Error>,
    ) -> Self {
        Self {
            pat,
            loc: loc.into(),
            ctx0,
            errors,
        }
    }
    pub fn remove_pat(&mut self) -> WithLoc {
        WithLoc::new(self.loc, self.ctx0, self.errors)
    }
    pub fn remove_pat_loc<'b>(&'b mut self) -> Simple<'b> {
        Simple::new(self.ctx0, self.errors)
    }
    pub fn set_pat(
        &mut self,
        pat: Option<&'a ir0::stmt::Pattern>,
    ) -> Option<&'a ir0::stmt::Pattern> {
        std::mem::replace(&mut self.pat, pat)
    }
}

/// A signals context from where components will get their inputs.
#[derive(Debug, PartialEq, Default)]
pub struct Flows {
    pub elements: HashMap<Ident, Typ>,
}

impl Flows {
    pub fn add_element(&mut self, element_name: Ident, element_type: &Typ) {
        match self.elements.insert(element_name, element_type.clone()) {
            Some(other_ty) => debug_assert!(other_ty.eq(element_type)),
            None => (),
        }
    }
    pub fn contains_element(&self, element_name: &Ident) -> bool {
        self.elements.contains_key(element_name)
    }

    pub fn into_syn(self) -> Vec<syn::Item> {
        // construct Context structure type
        let context_struct = {
            let fields = self.elements.iter().map(|(element_name, _)| -> syn::Field {
                let struct_name = element_name.to_camel();
                parse_quote! { pub #element_name: ctx_ty::#struct_name }
            });
            let name = Ident::new("Context", Span::call_site());
            let attribute: syn::Attribute =
                parse_quote!(#[derive(Clone, Copy, PartialEq, Default, Debug)]);
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
            parse_quote! { self.#element_name.reset(); }
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
                let struct_name = element_name.to_camel();
                let name = element_name;
                let ty = element_ty.into_syn();
                let attribute: syn::Attribute =
                    parse_quote!(#[derive(Clone, Copy, PartialEq, Default, Debug)]);
                let item_struct: syn::ItemStruct = parse_quote! {
                    #attribute
                    pub struct #struct_name(#ty, bool);
                };

                let item_impl: syn::ItemImpl = {
                    let set_impl: syn::ImplItem = parse_quote! {
                        pub fn set(&mut self, #name: #ty) {
                            self.1 = self.0 != #name;
                            self.0 = #name;
                        }
                    };
                    let get_impl: syn::ImplItem = parse_quote! {
                        pub fn get(&self) -> #ty {
                            self.0
                        }
                    };
                    let is_new_impl: syn::ImplItem = parse_quote! {
                        pub fn is_new(&self) -> bool {
                            self.1
                        }
                    };
                    let reset_impl: syn::ImplItem = parse_quote! {
                        pub fn reset(&mut self) {
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

        let types_mod = syn::Item::Mod(parse_quote! {
            mod ctx_ty {
                use super::*;

                #(#items)*
            }
        });

        vec![types_mod, context_struct, context_impl]
    }
}

pub struct Full<'a, Event> {
    pub imports: &'a mut HashMap<usize, interface::FlowImport>,
    pub exports: &'a HashMap<usize, interface::FlowExport>,
    pub timings: &'a mut Vec<Event>,
    pub ctx0: &'a mut Ctx,
}
impl<E> ops::Deref for Full<'_, E> {
    type Target = Ctx;
    fn deref(&self) -> &Self::Target {
        self.ctx0
    }
}
impl<E> ops::DerefMut for Full<'_, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx0
    }
}

mk_new! { impl{'a, Event} Full<'a, Event> => new {
    imports: &'a mut HashMap<usize, interface::FlowImport>,
    exports: &'a HashMap<usize, interface::FlowExport>,
    timings: &'a mut Vec<Event>,
    ctx0: &'a mut Ctx,
} }
