use crate::{
    backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    common::constant::Constant, lir::pattern::Pattern,
};
use proc_macro2::Span;
use quote::format_ident;
use syn::*;
/// Transform LIR pattern into RustAST pattern.
pub fn rust_ast_from_lir(pattern: Pattern) -> Pat {
    match pattern {
        Pattern::Literal { literal } => match literal {
            Constant::Integer(i) => Pat::Lit(parse_quote! { #i }),
            Constant::Float(f) => Pat::Lit(parse_quote! { #f }),
            Constant::Boolean(b) => Pat::Lit(parse_quote! { #b }),
            Constant::Unit(paren_token) => Pat::Tuple(PatTuple {
                attrs: vec![],
                paren_token,
                elems: Default::default(),
            }),
        },
        Pattern::Identifier { name } => Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new(&name, Span::call_site()),
            subpat: None,
        }),
        Pattern::Some { pattern } => Pat::TupleStruct(PatTupleStruct {
            attrs: vec![],
            path: parse_quote! { Some },
            elems: vec![rust_ast_from_lir(*pattern)].into_iter().collect(),
            paren_token: Default::default(),
            qself: None,
        }),
        Pattern::None => parse_quote! { None },
        Pattern::Default => Pat::Wild(PatWild {
            attrs: vec![],
            underscore_token: Default::default(),
        }),
        Pattern::Typed { pattern, typing } => Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(rust_ast_from_lir(*pattern)),
            colon_token: parse_quote!(:),
            ty: Box::new(type_rust_ast_from_lir(typing)),
        }),
        Pattern::Structure { name, fields } => Pat::Struct(PatStruct {
            attrs: vec![],
            path: format_ident!("{name}").into(),
            brace_token: Default::default(),
            fields: fields
                .into_iter()
                .map(|(name, pattern)| FieldPat {
                    attrs: vec![],
                    member: Member::Named(Ident::new(&name, Span::call_site())),
                    colon_token: Some(Default::default()),
                    pat: Box::new(rust_ast_from_lir(pattern)),
                })
                .collect(),
            qself: None,
            rest: None,
        }),
        Pattern::Enumeration {
            enum_name,
            elem_name,
        } => {
            let ty = Ident::new(&enum_name, Span::call_site());
            let cons = Ident::new(&elem_name, Span::call_site());

            Pat::Path(PatPath {
                attrs: vec![],
                qself: None,
                path: parse_quote! { #ty::#cons },
            })
        }
        Pattern::Tuple { elements } => Pat::Tuple(PatTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: elements
                .into_iter()
                .map(|element| rust_ast_from_lir(element))
                .collect(),
        }),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::pattern::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::lir::pattern::Pattern;
    use syn::*;

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_lir_default_pattern() {
        let pattern = Pattern::Default;
        let control = parse_quote! { _ };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_lir_none_pattern() {
        let pattern = Pattern::None;
        let control = parse_quote! { None };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_tuple_structure_pattern_from_a_lir_some_pattern() {
        let pattern = Pattern::Some {
            pattern: Box::new(Pattern::Default),
        };

        let control = parse_quote! { Some(_) };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_literal_pattern_from_a_lir_constant_pattern() {
        let pattern = Pattern::Literal {
            literal: Constant::Integer(parse_quote!(1)),
        };

        let control = parse_quote! { 1i64 };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_identifier_pattern_owned_and_immutable_from_a_lir_identifier_pattern(
    ) {
        let pattern = Pattern::Identifier {
            name: String::from("x"),
        };

        let control = parse_quote! { x };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_structure_pattern_from_a_lir_structure_pattern() {
        let pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (String::from("x"), Pattern::Default),
                (
                    String::from("y"),
                    Pattern::Identifier {
                        name: String::from("y"),
                    },
                ),
            ],
        };

        let control = parse_quote! { Point { x: _, y : y } };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }
}
