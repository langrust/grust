use crate::backend::rust_ast_from_lir::expression::{
    binary_to_syn, constant_to_syn, rust_ast_from_lir as expression_rust_ast_from_lir, unary_to_syn,
};
use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::statement::rust_ast_from_lir as statement_rust_ast_from_lir;
use crate::common::{convert_case::camel_case, r#type::Type as GRRustType, scope::Scope};
use crate::lir::contract::{Contract, Term};
use crate::lir::item::state_machine::state::step::{StateElementStep, Step};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::parse_quote;
use syn::*;

fn term_to_token_stream(term: Term, prophecy: bool) -> TokenStream {
    match term {
        Term::Unary { op, term } => {
            let ts_term = term_to_token_stream(*term, prophecy);
            let ts_op = unary_to_syn(op);
            quote!(#ts_op #ts_term)
        }
        Term::Binary { op, left, right } => {
            let ts_left = term_to_token_stream(*left, prophecy);
            let ts_right = term_to_token_stream(*right, prophecy);
            let ts_op = binary_to_syn(op);
            quote!(#ts_left #ts_op #ts_right)
        }
        Term::Constant { constant } => {
            let expr = constant_to_syn(constant);
            quote!(#expr)
        }
        Term::Identifier { name, scope } => {
            let id = Ident::new(&name, Span::call_site());
            match scope {
                Scope::Input => {
                    quote!(input.#id)
                }
                Scope::Memory => {
                    // there is prophecy only here
                    if prophecy {
                        quote!((^self).#id)
                    } else {
                        quote!(self.#id)
                    }
                }
                Scope::Output => quote!(result),
                Scope::Local => quote!(#id),
            }
        }
    }
}

/// Transform LIR step into RustAST implementation method.
pub fn rust_ast_from_lir(step: Step, crates: &mut BTreeSet<String>) -> ImplItemFn {
    let Contract {
        requires,
        ensures,
        invariant,
    } = step.contract;
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
            let ts_post = term_to_token_stream(term, true); // state postcondition
            vec![
                parse_quote!(#[requires(#ts_pre)]),
                parse_quote!(#[ensures(#ts_post)]),
            ]
        })
        .collect::<Vec<_>>();
    attributes.append(&mut ensures_attributes);
    attributes.append(&mut invariant_attributes);

    // create generics
    let mut generic_params: Vec<GenericParam> = vec![];
    let mut generic_idents: Vec<Ident> = vec![];
    for (generic_name, generic_type) in step.generics {
        if let GRRustType::Abstract(arguments, output) = generic_type {
            let arguments = arguments.into_iter().map(type_rust_ast_from_lir);
            let output = type_rust_ast_from_lir(*output);
            let identifier = format_ident!("{generic_name}");
            generic_params.push(parse_quote! { #identifier: Fn(#(#arguments),*) -> #output });
            generic_idents.push(identifier);
        } else {
            unreachable!()
        }
    }
    let generics = if generic_params.is_empty() {
        Default::default()
    } else {
        parse_quote! { <#(#generic_params),*> }
    };

    let input_ty_name = Ident::new(
        &camel_case(&format!("{}Input", step.node_name)),
        Span::call_site(),
    );
    let ty = if generic_idents.is_empty() {
        parse_quote! { #input_ty_name }
    } else {
        parse_quote! { #input_ty_name<#(#generic_idents),*> }
    };

    let inputs = vec![
        parse_quote!(&mut self),
        FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: Ident::new("input", Span::call_site()),
                subpat: None,
            })),
            colon_token: Default::default(),
            ty,
        }),
    ]
    .into_iter()
    .collect();

    let signature = syn::Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Default::default(),
        ident: Ident::new("step", Span::call_site()),
        generics,
        paren_token: Default::default(),
        inputs,
        variadic: None,
        output: ReturnType::Type(
            Default::default(),
            Box::new(type_rust_ast_from_lir(step.output_type)),
        ),
    };
    let mut statements = step
        .body
        .into_iter()
        .map(|statement| statement_rust_ast_from_lir(statement, crates))
        .collect::<Vec<_>>();

    let mut fields_update = step
        .state_elements_step
        .into_iter()
        .map(
            |StateElementStep {
                 identifier,
                 expression,
             }| {
                let identifier = Ident::new(&identifier, Span::call_site());
                let field_acces = parse_quote!(self.#identifier);

                Stmt::Expr(
                    Expr::Assign(ExprAssign {
                        attrs: vec![],
                        left: Box::new(field_acces),
                        eq_token: Default::default(),
                        right: Box::new(expression_rust_ast_from_lir(expression, crates)),
                    }),
                    Some(Default::default()),
                )
            },
        )
        .collect::<Vec<_>>();

    let output_statement = Stmt::Expr(
        expression_rust_ast_from_lir(step.output_expression, crates),
        None,
    );

    statements.append(&mut fields_update);
    statements.push(output_statement);

    let body = Block {
        stmts: statements,
        brace_token: Default::default(),
    };

    ImplItemFn {
        attrs: attributes,
        vis: Visibility::Public(Default::default()),
        defaultness: None,
        sig: signature,
        block: body,
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::item::state_machine::state::step::rust_ast_from_lir,
        common::{
            constant::Constant,
            r#type::Type,
        },
        lir::{
            expression::{Expression, FieldIdentifier},
            item::state_machine::state::step::{StateElementStep, Step},
            pattern::Pattern,
            statement::Statement,
        },
    }
    use syn::*;

    #[test]
    fn should_create_rust_ast_associated_method_from_lir_node_init() {
        let init = Step {
            contract: Default::default(),
            node_name: format!("Node"),
            generics: vec![],
            output_type: Type::Integer,
            body: vec![
                Statement::Let {
                    pattern: Pattern::ident("o"),
                    expression: Expression::field_access(
                        Expression::ident("self"),
                        FieldIdentifier::Named(format!("mem_i")),
                    ),
                },
                Statement::Let {
                    pattern: Pattern::Identifier {
                        name: String::from("y"),
                    },
                    expression: Expression::node_call(
                        "called_node_state",
                        "CalledNodeInput",
                        vec![],
                    ),
                },
            ],
            state_elements_step: vec![
                StateElementStep {
                    identifier: format!("mem_i"),
                    expression: Expression::binop(
                        crate::common::operator::BinaryOperator::Add,
                        Expression::ident("o"),
                        Expression::literal(Constant::Integer(parse_quote!(1i64))),
                    ),
                },
                StateElementStep {
                    identifier: format!("called_node_state"),
                    expression: Expression::Identifier {
                        identifier: format!("new_called_node_state"),
                    },
                },
            ],
            output_expression: Expression::binop(
                crate::common::operator::BinaryOperator::Add,
                Expression::ident("o"),
                Expression::ident("y"),
            ),
        };

        let control = parse_quote! {
            pub fn step(&mut self, input: NodeInput) -> i64 {
                let o = self.mem_i;
                let y = self.called_node_state.step(CalledNodeInput {});
                self.mem_i = o + 1i64;
                self.called_node_state = new_called_node_state;
                o + y
            }
        };
        assert_eq!(rust_ast_from_lir(init, &mut Default::default()), control)
    }
}
