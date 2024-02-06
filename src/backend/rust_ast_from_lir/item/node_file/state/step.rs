use crate::ast::expression;
use crate::ast::term::{Contract, Term};
use crate::backend::rust_ast_from_lir::expression::{binary_to_syn, rust_ast_from_lir as expression_rust_ast_from_lir};
use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::statement::rust_ast_from_lir as statement_rust_ast_from_lir;
use crate::common::convert_case::camel_case;
use crate::lir::item::node_file::state::step::{StateElementStep, Step};
use syn::token::Impl;
use syn::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

fn term_to_token_stream(term: Term) -> TokenStream {
    match term.kind {
        crate::ast::term::TermKind::Binary { op, left, right } => {
            let ts_left = term_to_token_stream(*left);
            let ts_right = term_to_token_stream(*right);
            let ts_op = binary_to_syn(op);
            quote!(#ts_left #ts_op #ts_right)
        }
        crate::ast::term::TermKind::Constant { constant } => {
            let s = format!("{constant}");
            s.parse().unwrap()
        }
        crate::ast::term::TermKind::Variable { id } => quote!(#id),
    }
}

/// Transform LIR step into RustAST implementation method.
pub fn rust_ast_from_lir(step: Step) -> ImplItemFn {
    let Contract { requires, ensures, .. } = step.contracts;
    let mut requires_attributes = requires
        .into_iter()
        .map(|term| {
            let ts = term_to_token_stream(term);
            parse_quote!(#[requires(#ts)])
        })
        .collect::<Vec<_>>();
    let mut attributes = ensures
        .into_iter()
        .map(|term| {
            let ts = term_to_token_stream(term);
            parse_quote!(#[ensures(#ts)])
        })
        .collect::<Vec<_>>();
    attributes.append(&mut requires_attributes);

    let input_ty_name = Ident::new(&camel_case(&format!("{}Input", step.node_name)), Span::call_site());
    let inputs = vec![FnArg::Typed(PatType{ 
        attrs: vec![],
        pat: Box::new(Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new("input", Span::call_site()),
            subpat: None,
        })),
        colon_token: Default::default(),
        ty: parse_quote!(#input_ty_name),
    })].into_iter().collect();

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
        .map(statement_rust_ast_from_lir)
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

                Stmt::Expr(Expr::Assign(ExprAssign {
                    attrs: vec![],
                    left: Box::new(field_acces),
                    eq_token: Default::default(),
                    right: Box::new(expression_rust_ast_from_lir(expression)),
                }), Some(Default::default()))
            },
        )
        .collect::<Vec<_>>();

    // let output_statement =
    //     Statement::ExpressionLast(expression_rust_ast_from_lir(step.output_expression));
    
    let output_statement = Stmt::Expr(expression_rust_ast_from_lir(step.output_expression), None);

    statements.append(&mut fields_update);
    statements.push(output_statement);

    let body = Block { stmts: statements, brace_token: Default::default() };

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
    use crate::backend::rust_ast_from_lir::item::node_file::state::step::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::lir::expression::{Expression, FieldIdentifier};
    use crate::lir::item::node_file::state::step::{StateElementStep, Step};
    use crate::lir::statement::Statement;
    use syn::*;

    #[test]
    fn should_create_rust_ast_associated_method_from_lir_node_init() {
        let init = Step {
            contracts: Default::default(),
            node_name: format!("Node"),
            output_type: Type::Integer,
            body: vec![
                Statement::Let {
                    identifier: format!("o"),
                    expression: Expression::FieldAccess {
                        expression: Box::new(Expression::Identifier {
                            identifier: format!("self"),
                        }),
                        field: FieldIdentifier::Named(format!("mem_i")),
                    },
                },
                Statement::Let {
                    identifier: format!("y"),
                    expression: Expression::NodeCall {
                        node_identifier: format!("called_node_state"),
                        input_name: format!("CalledNodeInput"),
                        input_fields: vec![],
                    },
                },
            ],
            state_elements_step: vec![
                StateElementStep {
                    identifier: format!("mem_i"),
                    expression: Expression::FunctionCall {
                        function: Box::new(Expression::Identifier {
                            identifier: format!(" + "),
                        }),
                        arguments: vec![
                            Expression::Identifier {
                                identifier: format!("o"),
                            },
                            Expression::Literal {
                                literal: Constant::Integer(1),
                            },
                        ],
                    },
                },
                StateElementStep {
                    identifier: format!("called_node_state"),
                    expression: Expression::Identifier {
                        identifier: format!("new_called_node_state"),
                    },
                },
            ],
            output_expression: Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    identifier: format!(" + "),
                }),
                arguments: vec![
                    Expression::Identifier {
                        identifier: format!("o"),
                    },
                    Expression::Identifier {
                        identifier: format!("y"),
                    },
                ],
            },
        };

        let control = parse_quote! {
            pub fn step(input: NodeInput) -> i64 {
                let o = self.mem_i;
                let y = self.called_node_state.step(CalledNodeInput {});
                self.mem_i = o + 1;
                self.called_node_state = new_called_node_state;
                o + y
            }
        };
        assert_eq!(rust_ast_from_lir(init), control)
    }
}
