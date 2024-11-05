use std::collections::BTreeSet;

prelude! {
    backend::rust_ast_from_lir::{
        expression::{
            binary_to_syn, constant_to_syn, unary_to_syn,
        },
        block::rust_ast_from_lir as block_rust_ast_from_lir,
        typ::rust_ast_from_lir as type_rust_ast_from_lir,
    },
    lir::{
        contract::{Contract, Term},
        item::function::Function},
    macro2::{Span, TokenStream},
    quote::quote,
}

fn term_to_token_stream(term: Term, prophecy: bool) -> TokenStream {
    match term {
        Term::Unop { op, term } => {
            let ts_term = term_to_token_stream(*term, prophecy);
            let ts_op = unary_to_syn(op);
            quote!(#ts_op #ts_term)
        }
        Term::Binop { op, left, right } => {
            let ts_left = term_to_token_stream(*left, prophecy);
            let ts_right = term_to_token_stream(*right, prophecy);
            let ts_op = binary_to_syn(op);
            quote!(#ts_left #ts_op #ts_right)
        }
        Term::Literal { literal } => {
            let expr = constant_to_syn(literal);
            quote!(#expr)
        }
        Term::Identifier { identifier } | Term::InputAccess { identifier } => {
            let id = Ident::new(&identifier, Span::call_site());
            quote!(#id)
        }
        Term::Implication { left, right } => {
            let ts_left = term_to_token_stream(*left, prophecy);
            let ts_right = term_to_token_stream(*right, prophecy);
            quote!(#ts_left ==> #ts_right)
        }
        Term::Forall { name, ty, term } => {
            let id = Ident::new(&name, Span::call_site());
            let ts_term = term_to_token_stream(*term, prophecy);
            let ts_ty = type_rust_ast_from_lir(ty);
            quote!(forall<#id:#ts_ty> #ts_term)
        }
        Term::Enumeration {
            enum_name,
            elem_name,
            element,
        } => {
            let ty = Ident::new(&enum_name, Span::call_site());
            let cons = Ident::new(&elem_name, Span::call_site());
            if let Some(term) = element {
                let inner = term_to_token_stream(*term, prophecy);
                parse_quote! { #ty::#cons(#inner) }
            } else {
                parse_quote! { #ty::#cons }
            }
        }
        Term::Ok { term } => {
            let ts_term = term_to_token_stream(*term, prophecy);
            parse_quote! { Ok(#ts_term) }
        }
        Term::Err => parse_quote! { Err(()) },
        Term::Some { term } => {
            let ts_term = term_to_token_stream(*term, prophecy);
            parse_quote! { Some(#ts_term) }
        }
        Term::None => parse_quote! { None },
        Term::MemoryAccess { .. } => unreachable!(),
    }
}

/// Transform LIR function into RustAST function.
pub fn rust_ast_from_lir(function: Function, crates: &mut BTreeSet<String>) -> syn::Item {
    // create attributes from contract
    let Contract {
        requires,
        ensures,
        invariant,
    } = function.contract;
    let mut attributes = requires
        .into_iter()
        .map(|term| {
            let ts = term_to_token_stream(term, false);
            parse_quote!(#[requires(#ts)])
        })
        .collect::<Vec<_>>();
    let mut ensures_attributes = ensures
        .into_iter()
        .map(|term| {
            let ts = term_to_token_stream(term, false);
            parse_quote!(#[ensures(#ts)])
        })
        .collect::<Vec<_>>();
    let mut invariant_attributes = invariant
        .into_iter()
        .flat_map(|term| {
            let ts_pre = term_to_token_stream(term.clone(), false);
            let ts_post = term_to_token_stream(term, true); // state post-condition
            vec![
                parse_quote!(#[requires(#ts_pre)]),
                parse_quote!(#[ensures(#ts_post)]),
            ]
        })
        .collect::<Vec<_>>();
    attributes.append(&mut ensures_attributes);
    attributes.append(&mut invariant_attributes);

    let inputs = function
        .inputs
        .into_iter()
        .map(|(name, typ)| {
            let name = Ident::new(&name, Span::call_site());
            syn::FnArg::Typed(syn::PatType {
                attrs: vec![],
                pat: parse_quote!(#name),
                colon_token: Default::default(),
                ty: Box::new(type_rust_ast_from_lir(typ)),
            })
        })
        .collect();

    let sig = syn::Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Default::default(),
        ident: Ident::new(&function.name, Span::call_site()),
        generics: Default::default(),
        paren_token: Default::default(),
        inputs,
        variadic: None,
        output: syn::ReturnType::Type(
            Default::default(),
            Box::new(type_rust_ast_from_lir(function.output)),
        ),
    };

    let item_function = syn::Item::Fn(syn::ItemFn {
        attrs: attributes,
        vis: syn::Visibility::Public(Default::default()),
        sig,
        block: Box::new(block_rust_ast_from_lir(function.body, crates)),
    });

    item_function
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::item::function::rust_ast_from_lir,
        lir::{ Block, item::Function, Stmt },
    }

    #[test]
    fn should_create_rust_ast_function_from_lir_function() {
        let function = Function {
            name: "foo".into(),
            inputs: vec![("a".into(), Typ::int()), ("b".into(), Typ::int())],
            output: Typ::int(),
            body: Block {
                statements: vec![Stmt::ExprLast {
                    expression: lir::Expr::binop(
                        operator::BinaryOperator::Add,
                        lir::Expr::ident("a"),
                        lir::Expr::ident("b"),
                    ),
                }],
            },
            contract: Default::default(),
        };

        let control = parse_quote! {
            pub fn foo(a: i64, b: i64) -> i64 {
                a + b
            }
        };
        assert_eq!(
            rust_ast_from_lir(function, &mut Default::default()),
            control
        )
    }
}
