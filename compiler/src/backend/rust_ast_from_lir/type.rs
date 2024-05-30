use crate::common::r#type::Type;
use proc_macro2::Span;
use syn::*;

/// Transform LIR type into RustAST type.
pub fn rust_ast_from_lir(r#type: Type) -> syn::Type {
    match r#type {
        Type::Integer => parse_quote!(i64),
        Type::Float => parse_quote!(f64),
        Type::Boolean => parse_quote!(bool),
        Type::Unit => parse_quote!(()),
        Type::Any => parse_quote!(std::any::Any),
        Type::Enumeration { name, .. } => {
            let identifier = Ident::new(&name, Span::call_site());
            parse_quote!(#identifier)
        }
        Type::Structure { name, .. } => {
            let identifier = Ident::new(&name, Span::call_site());
            parse_quote!(#identifier)
        }
        Type::Array(element, size) => {
            let ty = rust_ast_from_lir(*element);

            parse_quote!([#ty; #size])
        }
        Type::Abstract(arguments, output) => {
            let arguments = arguments.into_iter().map(rust_ast_from_lir);
            let output = rust_ast_from_lir(*output);
            parse_quote!(impl Fn(#(#arguments),*) -> #output)
        }
        Type::Tuple(elements) => {
            let tys = elements.into_iter().map(rust_ast_from_lir);

            parse_quote!((#(#tys),*))
        }
        Type::Generic(name) => {
            let identifier = Ident::new(&name, Span::call_site());
            parse_quote!(#identifier)
        }
        Type::Event(element) | Type::Signal(element) => rust_ast_from_lir(*element),
        Type::Timeout(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(Result<#ty, ()>)
        }
        Type::Time => parse_quote!(tokio::time::Interval),
        Type::SMEvent(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(#ty)
        }
        Type::SMTimeout(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(Result<#ty, ()>)
        }
        Type::ComponentEvent => todo!(),
        Type::NotDefinedYet(_) | Type::Polymorphism(_) => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::r#type::rust_ast_from_lir,
        common::r#type::Type,
    }
    use syn::*;

    #[test]
    fn should_create_rust_ast_owned_i64_from_lir_integer() {
        let r#type = Type::Integer;
        let control = parse_quote! { i64 };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_f64_from_lir_float() {
        let r#type = Type::Float;
        let control = parse_quote! { f64 };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_bool_from_lir_boolean() {
        let r#type = Type::Boolean;
        let control = parse_quote! { bool };
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_unit_from_lir_unit() {
        let r#type = Type::Unit;
        let control = parse_quote! { () };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_structure_from_lir_structure() {
        let r#type = Type::Structure {
            name: String::from("Point"),
            id: 0,
        };
        let control = parse_quote! { Point };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_enumeration_from_lir_enumeration() {
        let r#type = Type::Enumeration {
            name: String::from("Color"),
            id: 0,
        };
        let control = parse_quote! { Color };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_array_from_lir_array() {
        let r#type = Type::Array(Box::new(Type::Float), 5);
        let control = parse_quote! { [f64;5usize] };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_generic_from_lir_option() {
        let r#type = Type::SMEvent(Box::new(Type::Float));
        let control = parse_quote!(Option<f64>);
        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_closure_from_lir_abstract() {
        let r#type = Type::Abstract(vec![Type::Integer], Box::new(Type::Float));
        let control = parse_quote!(impl Fn(i64) -> f64);

        assert_eq!(rust_ast_from_lir(r#type), control)
    }
}
