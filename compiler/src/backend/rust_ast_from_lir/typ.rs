prelude! {
    syn::{LitInt},
}

/// Transform LIR type into RustAST type.
pub fn rust_ast_from_lir(typ: Typ) -> syn::Type {
    match typ {
        Typ::Integer(_) => parse_quote!(i64),
        Typ::Float(_) => parse_quote!(f64),
        Typ::Boolean(_) => parse_quote!(bool),
        Typ::Unit(_) => parse_quote!(()),
        Typ::Enumeration { name, .. } => {
            parse_quote!(#name)
        }
        Typ::Structure { name, .. } => {
            parse_quote!(#name)
        }
        Typ::Array { ty, size, .. } => {
            let ty = rust_ast_from_lir(*ty);
            let size = syn::Lit::Int(LitInt::new(
                &(size.base10_digits().to_owned() + "usize"),
                size.span(),
            ));

            parse_quote!([#ty; #size])
        }
        Typ::Abstract { inputs, output, .. } => {
            let arguments = inputs.into_iter().map(rust_ast_from_lir);
            let output = rust_ast_from_lir(*output);
            parse_quote!(impl Fn(#(#arguments),*) -> #output)
        }
        Typ::Tuple { elements, .. } => {
            let tys = elements.into_iter().map(rust_ast_from_lir);

            parse_quote!((#(#tys),*))
        }
        Typ::Event { ty, .. } | Typ::Signal { ty, .. } => rust_ast_from_lir(*ty),
        Typ::SMEvent { ty, .. } => {
            let ty = rust_ast_from_lir(*ty);
            parse_quote!(Option<#ty>)
        }
        Typ::NotDefinedYet(_) | Typ::Polymorphism(_) | Typ::Any => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::typ::rust_ast_from_lir,
        syn::parse_quote,
    }

    #[test]
    fn should_create_i64_from_lir_integer() {
        let typ = Typ::int();
        let control = parse_quote! { i64 };
        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_f64_from_lir_float() {
        let typ = Typ::float();
        let control = parse_quote! { f64 };
        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_bool_from_lir_boolean() {
        let typ = Typ::bool();
        let control = parse_quote! { bool };
        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_unit_from_lir_unit() {
        let typ = Typ::unit();
        let control = parse_quote! { () };

        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_structure_from_lir_structure() {
        let typ = Typ::structure("Point", 0);
        let control = parse_quote! { Point };

        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_enumeration_from_lir_enumeration() {
        let typ = Typ::enumeration("Color", 0);
        let control = parse_quote! { Color };

        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_array_from_lir_array() {
        let typ = Typ::array(Typ::float(), 5);
        let control = parse_quote! { [f64; 5usize] };

        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_option_from_lir_statemachine_event() {
        let typ = Typ::sm_event(Typ::float());
        let control = parse_quote!(Option<f64>);
        assert_eq!(rust_ast_from_lir(typ), control)
    }

    #[test]
    fn should_create_closure_from_lir_abstract() {
        let typ = Typ::function(vec![Typ::int()], Typ::float());
        let control = parse_quote!(impl Fn(i64) -> f64);

        assert_eq!(rust_ast_from_lir(typ), control)
    }
}
