prelude! {
    macro2::Span,
    syn::*,
}

/// Transform LIR type into RustAST type.
pub fn rust_ast_from_lir(r#type: Typ) -> Type {
    match r#type {
        Typ::Integer => parse_quote!(i64),
        Typ::Float => parse_quote!(f64),
        Typ::Boolean => parse_quote!(bool),
        Typ::Unit => parse_quote!(()),
        Typ::Any => parse_quote!(std::any::Any),
        Typ::Enumeration { name, .. } => {
            let identifier = Ident::new(&name, Span::call_site());
            parse_quote!(#identifier)
        }
        Typ::Structure { name, .. } => {
            let identifier = Ident::new(&name, Span::call_site());
            parse_quote!(#identifier)
        }
        Typ::Array(element, size) => {
            let ty = rust_ast_from_lir(*element);

            parse_quote!([#ty; #size])
        }
        Typ::Abstract(arguments, output) => {
            let arguments = arguments.into_iter().map(rust_ast_from_lir);
            let output = rust_ast_from_lir(*output);
            parse_quote!(impl Fn(#(#arguments),*) -> #output)
        }
        Typ::Tuple(elements) => {
            let tys = elements.into_iter().map(rust_ast_from_lir);

            parse_quote!((#(#tys),*))
        }
        Typ::Generic(name) => {
            let identifier = Ident::new(&name, Span::call_site());
            parse_quote!(#identifier)
        }
        Typ::Event(element) | Typ::Signal(element) => rust_ast_from_lir(*element),
        Typ::Timeout(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(Result<#ty, ()>)
        }
        Typ::Time => parse_quote!(tokio::time::Interval),
        Typ::SMEvent(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(#ty)
        }
        Typ::SMTimeout(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(Result<#ty, ()>)
        }
        Typ::ComponentEvent => todo!(),
        Typ::NotDefinedYet(_) | Typ::Polymorphism(_) => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::r#type::rust_ast_from_lir,
        syn::parse_quote,
    }

    #[test]
    fn should_create_rust_ast_owned_i64_from_lir_integer() {
        let r#type = Typ::int();
        let control = parse_quote! { i64 };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_f64_from_lir_float() {
        let r#type = Typ::float();
        let control = parse_quote! { f64 };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_bool_from_lir_boolean() {
        let r#type = Typ::bool();
        let control = parse_quote! { bool };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_unit_from_lir_unit() {
        let r#type = Typ::unit();
        let control = parse_quote! { () };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_structure_from_lir_structure() {
        let r#type = Typ::structure("Point", 0);
        let control = parse_quote! { Point };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_enumeration_from_lir_enumeration() {
        let r#type = Typ::enumeration("Color", 0);
        let control = parse_quote! { Color };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_array_from_lir_array() {
        let r#type = Typ::array(Typ::float(), 5);
        let control = parse_quote! { [f64; 5usize] };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    // // #TODO come back to this test when we have options proper
    // #[test]
    // fn should_create_rust_ast_owned_generic_from_lir_option() {
    //     let r#type = Typ::sm_event(Typ::float());
    //     let control = parse_quote!(Option<f64>);
    //     assert_eq!(rust_ast_from_lir(r#type), control)
    // }

    #[test]
    fn should_create_rust_ast_owned_closure_from_lir_abstract() {
        let r#type = Typ::function(vec![Typ::Integer], Typ::float());
        let control = parse_quote!(impl Fn(i64) -> f64);

        assert_eq!(rust_ast_from_lir(r#type), control)
    }
}
