use crate::common::r#type::Type;
use proc_macro2::Span;
use syn::*;

/// Transform LIR type into RustAST type.
pub fn rust_ast_from_lir(r#type: Type) -> syn::Type {
    match r#type {
        Type::Integer => parse_quote!(i64),
        Type::Float => parse_quote!(f64),
        Type::Boolean => parse_quote!(bool),
        Type::String => parse_quote!(String),
        Type::Unit => parse_quote!(()),
        Type::Enumeration(identifier) => {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote!(#identifier)
        }
        Type::Structure(identifier) =>  {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote!(#identifier)
        },
        Type::Array(element, size) => {
            let ty = rust_ast_from_lir(*element);

            parse_quote!([#ty; #size])
        }
        Type::Option(element) => {
            let ty = rust_ast_from_lir(*element);
            parse_quote!(Option<#ty>)
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
        Type::NotDefinedYet(_) | Type::Polymorphism(_) => unreachable!(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir;
    use crate::common::r#type::Type;
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
    fn should_create_rust_ast_owned_string_from_lir_string() {
        let r#type = Type::String;
        let control = parse_quote! { String };

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
        let r#type = Type::Structure(String::from("Point"));
        let control = parse_quote! { Point };

        assert_eq!(rust_ast_from_lir(r#type), control)
    }

    #[test]
    fn should_create_rust_ast_owned_enumeration_from_lir_enumeration() {
        let r#type = Type::Enumeration(String::from("Color"));
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
        let r#type = Type::Option(Box::new(Type::Float));
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
