use std::collections::BTreeSet;

prelude! {
    backend::rust_ast_from_lir::{
        block::rust_ast_from_lir as block_rust_ast_from_lir,
        r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    },
    lir::item::function::Function,
    macro2::Span,
    quote::format_ident,
    syn::*,
}

/// Transform LIR function into RustAST function.
pub fn rust_ast_from_lir(function: Function, crates: &mut BTreeSet<String>) -> Item {
    // create generics
    let mut generic_params: Vec<GenericParam> = vec![];
    for (generic_name, generic_type) in function.generics {
        if let Typ::Abstract(arguments, output) = generic_type {
            let arguments = arguments.into_iter().map(type_rust_ast_from_lir);
            let output = type_rust_ast_from_lir(*output);
            let identifier = format_ident!("{generic_name}");
            generic_params.push(parse_quote! { #identifier: Fn(#(#arguments),*) -> #output });
        } else {
            unreachable!()
        }
    }
    let generics = if generic_params.is_empty() {
        Default::default()
    } else {
        parse_quote! { <#(#generic_params),*> }
    };

    let inputs = function
        .inputs
        .into_iter()
        .map(|(name, r#type)| {
            let name = Ident::new(&name, Span::call_site());
            FnArg::Typed(PatType {
                attrs: vec![],
                pat: parse_quote!(#name),
                colon_token: Default::default(),
                ty: Box::new(type_rust_ast_from_lir(r#type)),
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
        generics,
        paren_token: Default::default(),
        inputs,
        variadic: None,
        output: ReturnType::Type(
            Default::default(),
            Box::new(type_rust_ast_from_lir(function.output)),
        ),
    };

    let item_function = Item::Fn(ItemFn {
        attrs: Default::default(),
        vis: Visibility::Public(Default::default()),
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
    use syn::*;

    #[test]
    fn should_create_rust_ast_function_from_lir_function() {
        let function = Function {
            name: "foo".into(),
            generics: vec![],
            inputs: vec![("a".into(), Typ::Integer), ("b".into(), Typ::Integer)],
            output: Typ::Integer,
            body: Block {
                statements: vec![Stmt::ExprLast {
                    expression: lir::Expr::binop(
                        operator::BinaryOperator::Add,
                        lir::Expr::ident("a"),
                        lir::Expr::ident("b"),
                    ),
                }],
            },
            imports: vec![],
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
