use std::collections::BTreeSet;

prelude! {
    macro2::{Span, TokenStream},
    quote::quote,
    syn::*,
    backend::rust_ast_from_lir::{
        expression::{
            binary_to_syn, constant_to_syn, rust_ast_from_lir as expression_rust_ast_from_lir, unary_to_syn,
        },
        r#type::rust_ast_from_lir as type_rust_ast_from_lir,
        statement::rust_ast_from_lir as statement_rust_ast_from_lir,
    },
    lir::{
        contract::{Contract, Term},
        item::state_machine::state::step::{StateElementStep, Step},
    },
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
        Term::Identifier { identifier } => {
            let id = Ident::new(&identifier, Span::call_site());
            quote!(#id)
        }
        Term::MemoryAccess { identifier } => {
            let id = Ident::new(&identifier, Span::call_site());
            if prophecy {
                quote!((^self).#id)
            } else {
                quote!(self.#id)
            }
        }
        Term::InputAccess { identifier } => {
            let id = Ident::new(&identifier, Span::call_site());
            quote!(input.#id)
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
    }
}

/// Transform LIR step into RustAST implementation method.
pub fn rust_ast_from_lir(step: Step, crates: &mut BTreeSet<String>) -> ImplItemFn {
    // create attributes from contract
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
            let ts_post = term_to_token_stream(term, true); // state post-condition
            vec![
                parse_quote!(#[requires(#ts_pre)]),
                parse_quote!(#[ensures(#ts_post)]),
            ]
        })
        .collect::<Vec<_>>();
    attributes.append(&mut ensures_attributes);
    attributes.append(&mut invariant_attributes);

    let input_ty_name = Ident::new(
        &to_camel_case(&format!("{}Input", step.node_name)),
        Span::call_site(),
    );
    let ty = parse_quote! { #input_ty_name };

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
        generics: Default::default(),
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
        syn::*,
        backend::rust_ast_from_lir::item::state_machine::state::step::rust_ast_from_lir,
        lir::{
            FieldIdentifier, Pattern, Stmt,
            item::state_machine::state::step::{StateElementStep, Step},
        },
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_lir_node_init() {
        let init = Step {
            contract: Default::default(),
            node_name: format!("Node"),
            output_type: Typ::int(),
            body: vec![
                Stmt::Let {
                    pattern: Pattern::ident("o"),
                    expression: lir::Expr::field_access(
                        lir::Expr::ident("self"),
                        FieldIdentifier::named("mem_i"),
                    ),
                },
                Stmt::Let {
                    pattern: Pattern::ident("y"),
                    expression: lir::Expr::node_call(
                        "called_node_state",
                        "called_node",
                        "CalledNodeInput",
                        vec![],
                    ),
                },
            ],
            state_elements_step: vec![
                StateElementStep::new(
                    "mem_i",
                    lir::Expr::binop(
                        operator::BinaryOperator::Add,
                        lir::Expr::ident("o"),
                        lir::Expr::lit(Constant::Integer(parse_quote!(1i64))),
                    ),
                ),
                StateElementStep::new(
                    "called_node_state",
                    lir::Expr::ident("new_called_node_state"),
                ),
            ],
            output_expression: lir::Expr::binop(
                operator::BinaryOperator::Add,
                lir::Expr::ident("o"),
                lir::Expr::ident("y"),
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
