use crate::ast::pattern::Pattern;
use proc_macro2::Span;
use quote::format_ident;
use syn::*;
/// Transform LIR pattern into RustAST pattern.
pub fn rust_ast_from_lir(pattern: Pattern) -> Pat {
    match pattern {
        Pattern::Identifier { name, location: _ } => Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new(&name, Span::call_site()),
            subpat: None,
        }),
        Pattern::Some {
            pattern,
            location: _,
        } => Pat::TupleStruct(PatTupleStruct {
            attrs: vec![],
            path: parse_quote! { Some },
            elems: vec![rust_ast_from_lir(*pattern)].into_iter().collect(),
            paren_token: Default::default(),
            qself: None,
        }),
        Pattern::None { location: _ } => parse_quote! { None },
        Pattern::Default { location: _ } => Pat::Wild(PatWild {
            attrs: vec![],
            underscore_token: Default::default(),
        }),
        Pattern::Structure {
            name,
            fields,
            location: _,
        } => Pat::Struct(PatStruct {
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
        Pattern::Tuple {
            elements,
            location: _,
        } => Pat::Tuple(PatTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: elements.into_iter().map(rust_ast_from_lir).collect(),
        }),

        Pattern::Constant {
            constant,
            location: _,
        } => match constant {
            crate::common::constant::Constant::Integer(i) => Pat::Lit(parse_quote! { #i }),
            crate::common::constant::Constant::Float(f) => Pat::Lit(parse_quote! { #f }),
            crate::common::constant::Constant::Boolean(b) => Pat::Lit(parse_quote! { #b }),
            crate::common::constant::Constant::String(s) => Pat::Lit(parse_quote! { #s }),
            crate::common::constant::Constant::Unit => Pat::Tuple(PatTuple {
                attrs: vec![],
                paren_token: Default::default(),
                elems: Default::default(),
            }),
            crate::common::constant::Constant::Enumeration(ty, cons) => {
                let ty = Ident::new(&ty, Span::call_site());
                let cons = Ident::new(&cons, Span::call_site());

                Pat::Path(PatPath {
                    attrs: vec![],
                    qself: None,
                    path: parse_quote! { #ty::#cons },
                })
            }
        },
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::ast::pattern::Pattern;
    use crate::backend::rust_ast_from_lir::pattern::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::common::location::Location;
    use syn::*;

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_lir_default_pattern() {
        let pattern = Pattern::Default {
            location: Location::default(),
        };
        let control = parse_quote! { _ };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_lir_none_pattern() {
        let pattern = Pattern::None {
            location: Location::default(),
        };
        let control = parse_quote! { None };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_tuple_structure_pattern_from_a_lir_some_pattern() {
        let pattern = Pattern::Some {
            pattern: Box::new(Pattern::Default {
                location: Location::default(),
            }),
            location: Location::default(),
        };

        let control = parse_quote! { Some(_) };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_literal_pattern_from_a_lir_constant_pattern() {
        let pattern = Pattern::Constant {
            constant: Constant::Integer(1),
            location: Location::default(),
        };

        let control = parse_quote! { 1i64 };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_identifier_pattern_owned_and_immutable_from_a_lir_identifier_pattern(
    ) {
        let pattern = Pattern::Identifier {
            name: String::from("x"),
            location: Location::default(),
        };

        let control = parse_quote! { x };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }

    #[test]
    fn should_create_a_rust_ast_structure_pattern_from_a_lir_structure_pattern() {
        let pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Pattern::Default {
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Pattern::Identifier {
                        name: String::from("y"),
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let control = parse_quote! { Point { x: _, y : y } };
        assert_eq!(rust_ast_from_lir(pattern), control)
    }
}
