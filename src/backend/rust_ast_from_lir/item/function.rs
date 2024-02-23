use crate::backend::rust_ast_from_lir::{
    block::rust_ast_from_lir as block_rust_ast_from_lir,
    item::import::rust_ast_from_lir as import_rust_ast_from_lir,
    r#type::rust_ast_from_lir as type_rust_ast_from_lir,
};
use crate::common::r#type::Type as GRRustType;
use crate::lir::item::function::Function;
use proc_macro2::Span;
use quote::format_ident;
use std::collections::BTreeSet;
use syn::*;

/// Transform LIR function into RustAST function.
pub fn rust_ast_from_lir(function: Function, crates: &mut BTreeSet<String>) -> Vec<Item> {
    let mut items = function
        .imports
        .into_iter()
        .map(|import| Item::Use(import_rust_ast_from_lir(import, crates)))
        .collect::<Vec<_>>();

    // create generics
    let mut generic_params: Vec<GenericParam> = vec![];
    for (generic_name, generic_type) in function.generics {
        if let GRRustType::Abstract(arguments, output) = generic_type {
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
    items.push(item_function);

    items
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::function::rust_ast_from_lir;
    use crate::common::r#type::Type;
    use crate::lir::block::Block;
    use crate::lir::expression::Expression;
    use crate::lir::item::function::Function;
    use crate::lir::statement::Statement;
    use syn::*;

    #[test]
    fn should_create_rust_ast_function_from_lir_function() {
        let function = Function {
            name: String::from("foo"),
            generics: vec![],
            inputs: vec![
                (String::from("a"), Type::Integer),
                (String::from("b"), Type::Integer),
            ],
            output: Type::Integer,
            body: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::FunctionCall {
                        function: Box::new(Expression::Identifier {
                            identifier: String::from(" + "),
                        }),
                        arguments: vec![
                            Expression::Identifier {
                                identifier: String::from("a"),
                            },
                            Expression::Identifier {
                                identifier: String::from("b"),
                            },
                        ],
                    },
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
            vec![control]
        )
    }
}
