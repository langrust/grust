use super::{
    block::rust_ast_from_lir as block_rust_ast_from_lir,
    pattern::rust_ast_from_lir as pattern_rust_ast_from_lir,
    r#type::rust_ast_from_lir as type_rust_ast_from_lir,
};
use crate::common::operator::{BinaryOperator, UnaryOperator};
use crate::lir::expression::{Expression, FieldIdentifier};
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use std::collections::BTreeSet;
use strum::IntoEnumIterator;
use syn::*;

/// Transforms binary operator into syn's binary operator.
pub fn binary_to_syn(op: BinaryOperator) -> syn::BinOp {
    match op {
        BinaryOperator::Mul => BinOp::Mul(Default::default()),
        BinaryOperator::Div => BinOp::Div(Default::default()),
        BinaryOperator::Add => BinOp::Add(Default::default()),
        BinaryOperator::Sub => BinOp::Sub(Default::default()),
        BinaryOperator::And => BinOp::And(Default::default()),
        BinaryOperator::Or => BinOp::Or(Default::default()),
        BinaryOperator::Eq => BinOp::Eq(Default::default()),
        BinaryOperator::Dif => BinOp::Ne(Default::default()),
        BinaryOperator::Geq => BinOp::Ge(Default::default()),
        BinaryOperator::Leq => BinOp::Le(Default::default()),
        BinaryOperator::Grt => BinOp::Gt(Default::default()),
        BinaryOperator::Low => BinOp::Lt(Default::default()),
    }
}

/// Transforms unary operator into syn's unary operator.
pub fn unary_to_syn(op: UnaryOperator) -> Option<syn::UnOp> {
    match op {
        UnaryOperator::Neg => Some(UnOp::Neg(Default::default())),
        UnaryOperator::Not => Some(UnOp::Not(Default::default())),
        UnaryOperator::Brackets => None,
    }
}

/// Transform LIR expression into RustAST expression.
pub fn rust_ast_from_lir(expression: Expression, crates: &mut BTreeSet<String>) -> Expr {
    match expression {
        Expression::Literal { literal } => syn::parse_str(&format!("{literal}")).unwrap(),
        Expression::Identifier { identifier } => {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote! { #identifier }
        }
        Expression::MemoryAccess { identifier } => {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote!( self . #identifier )
        }
        Expression::InputAccess { identifier } => {
            let identifier = format_ident!("{identifier}");
            parse_quote!( input . #identifier )
        }
        Expression::Structure { name, fields } => {
            let fields: Vec<FieldValue> = fields
                .into_iter()
                .map(|(name, expression)| {
                    let name = format_ident!("{name}");
                    let expression = rust_ast_from_lir(expression, crates);
                    parse_quote!(#name : #expression)
                })
                .collect();
            let name = format_ident!("{name}");
            parse_quote!(#name { #(#fields),* })
        }
        Expression::Enumeration { name, element } => {
            syn::parse_str(&format!("{name}::{element}")).unwrap()
        }
        Expression::Array { elements } => {
            let elements = elements
                .into_iter()
                .map(|expression| rust_ast_from_lir(expression, crates));
            parse_quote! { [#(#elements),*]}
        }
        Expression::Tuple { elements } => {
            let elements = elements
                .into_iter()
                .map(|expression| rust_ast_from_lir(expression, crates));
            parse_quote! { (#(#elements),*)}
        }
        Expression::Block { block } => Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: block_rust_ast_from_lir(block, crates),
        }),
        Expression::FunctionCall {
            function,
            mut arguments,
        } => match function.as_ref() {
            Expression::Identifier { identifier } => {
                if let Some(binary) =
                    BinaryOperator::iter().find(|binary| binary.to_string() == *identifier)
                {
                    let left = rust_ast_from_lir(arguments.remove(0), crates);
                    let right = rust_ast_from_lir(arguments.remove(0), crates);
                    let binary = binary_to_syn(binary);
                    Expr::Binary(parse_quote! { #left #binary #right })
                } else if let Some(unary) =
                    UnaryOperator::iter().find(|unary| unary.to_string() == *identifier)
                {
                    let op = unary_to_syn(unary);
                    let expr = rust_ast_from_lir(arguments.remove(0), crates);
                    if let Some(op) = op {
                        Expr::Unary(parse_quote! { #op #expr})
                    } else {
                        Expr::Paren(ExprParen {
                            attrs: vec![],
                            paren_token: Default::default(),
                            expr: Box::new(expr),
                        })
                    }
                } else {
                    let function = rust_ast_from_lir(*function, crates);
                    let arguments = arguments
                        .into_iter()
                        .map(|expression| rust_ast_from_lir(expression, crates));
                    parse_quote! {
                        #function (#(#arguments),*)
                    }
                }
            }
            _ => {
                let function = rust_ast_from_lir(*function, crates);
                let arguments = arguments
                    .into_iter()
                    .map(|expression| rust_ast_from_lir(expression, crates));
                parse_quote! { (#function)(#(#arguments),*) }
            }
        },
        Expression::NodeCall {
            node_identifier,
            input_name,
            input_fields,
        } => {
            let id = Ident::new(&node_identifier, Span::call_site());
            let receiver: ExprField = parse_quote! { self . #id};
            let input_fields: Vec<FieldValue> = input_fields
                .into_iter()
                .map(|(name, expression)| {
                    let id = Ident::new(&name, Span::call_site());
                    let expr = rust_ast_from_lir(expression, crates);
                    parse_quote! { #id : #expr }
                })
                .collect();

            let input_name = Ident::new(&input_name, Span::call_site());
            let argument: ExprStruct = parse_quote! { #input_name { #(#input_fields),* }};

            Expr::MethodCall(parse_quote! { #receiver . step (#argument) })
        }
        Expression::FieldAccess { expression, field } => {
            let expression = rust_ast_from_lir(*expression, crates);
            match field {
                FieldIdentifier::Named(name) => {
                    let name = Ident::new(&name, Span::call_site());
                    parse_quote!(#expression . #name)
                }
                FieldIdentifier::Unamed(number) => {
                    let number: TokenStream = format!("{number}").parse().unwrap();
                    parse_quote!(#expression . #number)
                }
            }
        }
        Expression::Lambda {
            inputs,
            output,
            body,
        } => {
            let inputs = inputs
                .into_iter()
                .map(|(identifier, r#type)| {
                    let pattern = syn::Pat::Ident(syn::PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: syn::Ident::new(&identifier, proc_macro2::Span::call_site()),
                        subpat: None,
                    });
                    let pattern = syn::Pat::Type(syn::PatType {
                        attrs: Vec::new(),
                        pat: Box::new(pattern),
                        colon_token: Default::default(),
                        ty: Box::new(type_rust_ast_from_lir(r#type)),
                    });
                    pattern
                })
                .collect();
            let closure = syn::ExprClosure {
                attrs: Vec::new(),
                asyncness: None,
                movability: None,
                capture: Some(syn::token::Move {
                    span: proc_macro2::Span::call_site(),
                }),
                or1_token: Default::default(),
                inputs,
                or2_token: Default::default(),
                output: ReturnType::Type(
                    Default::default(),
                    Box::new(type_rust_ast_from_lir(output)),
                ),
                body: Box::new(rust_ast_from_lir(*body, crates)),
                lifetimes: None,
                constness: None,
            };
            syn::Expr::Closure(closure)
        }
        Expression::IfThenElse {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = Box::new(rust_ast_from_lir(*condition, crates));
            let then_branch = block_rust_ast_from_lir(then_branch, crates);
            let else_branch = block_rust_ast_from_lir(else_branch, crates);
            let else_branch = parse_quote! { #else_branch };
            let else_branch = Some((Default::default(), Box::new(else_branch)));
            syn::Expr::If(syn::ExprIf {
                attrs: Vec::new(),
                if_token: Default::default(),
                cond: condition,
                then_branch,
                else_branch,
            })
        }
        Expression::Match { matched, arms } => {
            let arms = arms
                .into_iter()
                .map(|(pattern, guard, body)| syn::Arm {
                    attrs: Vec::new(),
                    pat: pattern_rust_ast_from_lir(pattern),
                    guard: guard
                        .map(|expression| rust_ast_from_lir(expression, crates))
                        .map(|g| (Default::default(), Box::new(g))),
                    body: Box::new(rust_ast_from_lir(body, crates)),
                    fat_arrow_token: Default::default(),
                    comma: Some(Default::default()),
                })
                .collect();
            syn::Expr::Match(syn::ExprMatch {
                attrs: Vec::new(),
                match_token: Default::default(),
                expr: Box::new(rust_ast_from_lir(*matched, crates)),
                brace_token: Default::default(),
                arms,
            })
        }
        Expression::Map { mapped, function } => {
            let receiver = Box::new(rust_ast_from_lir(*mapped, crates));
            let method = syn::Ident::new("map", proc_macro2::Span::call_site());
            let arguments = vec![rust_ast_from_lir(*function, crates)];
            let method_call = syn::ExprMethodCall {
                attrs: Vec::new(),
                receiver,
                method,
                turbofish: None,
                paren_token: Default::default(),
                args: syn::punctuated::Punctuated::from_iter(arguments),
                dot_token: Default::default(),
            };
            syn::Expr::MethodCall(method_call)
        }
        Expression::Fold {
            folded,
            initialization,
            function,
        } => syn::Expr::MethodCall(syn::ExprMethodCall {
            attrs: Vec::new(),
            receiver: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs: Vec::new(),
                receiver: Box::new(rust_ast_from_lir(*folded, crates)),
                method: syn::Ident::new("into_iter", proc_macro2::Span::call_site()),
                turbofish: None,
                paren_token: Default::default(),
                args: syn::punctuated::Punctuated::new(),
                dot_token: Default::default(),
            })),
            method: syn::Ident::new("fold", proc_macro2::Span::call_site()),
            turbofish: None,
            paren_token: Default::default(),
            args: syn::punctuated::Punctuated::from_iter(vec![
                rust_ast_from_lir(*initialization, crates),
                rust_ast_from_lir(*function, crates),
            ]),
            dot_token: Default::default(),
        }),
        Expression::Sort { sorted, function } => {
            let token_sorted = rust_ast_from_lir(*sorted, crates);
            let token_function = rust_ast_from_lir(*function, crates);

            parse_quote!({
                let mut x = #token_sorted.clone();
                let slice = x.as_mut();
                slice.sort_by(|a, b| {
                    let compare = #token_function(*a, *b);
                    if compare < 0 { std::cmp::Ordering::Less }
                    else if compare > 0 { std::cmp::Ordering::Greater }
                    else { std::cmp::Ordering::Equal }
                });
                x
            })
        }
        Expression::Zip { arrays } => {
            crates.insert(String::from("itertools = \"0.12.1\""));
            let arguments = arrays
                .into_iter()
                .map(|expression| rust_ast_from_lir(expression, crates));
            let macro_call = syn::ExprMacro {
                attrs: Vec::new(),
                mac: syn::Macro {
                    path: parse_quote!(itertools::izip),
                    bang_token: Default::default(),
                    delimiter: syn::MacroDelimiter::Paren(Default::default()),
                    tokens: quote::quote! { #(#arguments),* },
                },
            };
            let izip = syn::Expr::Macro(macro_call);

            parse_quote!({
                let mut iter = #izip;
                std::array::from_fn(|_| iter.next().unwrap())
            })
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::expression::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::common::r#type::Type;
    use crate::lir::block::Block;
    use crate::lir::expression::{Expression, FieldIdentifier};
    use crate::lir::pattern::Pattern;
    use crate::lir::statement::Statement;
    use syn::*;
    #[test]
    fn should_create_rust_ast_literal_from_lir_literal() {
        let expression = Expression::Literal {
            literal: Constant::Integer(1),
        };
        let control = parse_quote! { 1i64 };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_identifier_from_lir_identifier() {
        let expression = Expression::Identifier {
            identifier: String::from("x"),
        };
        let control = parse_quote! { x };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_field_access_to_self_from_lir_memory_access() {
        let expression = Expression::MemoryAccess {
            identifier: String::from("mem_x"),
        };
        let control = parse_quote! { self . mem_x};
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_field_access_to_input_from_lir_input_access() {
        let expression = Expression::InputAccess {
            identifier: String::from("i"),
        };
        let control = parse_quote! { input . i};
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_structure_from_lir_structure() {
        let expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
                (
                    String::from("y"),
                    Expression::Literal {
                        literal: Constant::Integer(2),
                    },
                ),
            ],
        };
        let control = parse_quote! { Point { x : 1i64, y : 2i64 } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_array_from_lir_array() {
        let expression = Expression::Array {
            elements: vec![
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
                Expression::Literal {
                    literal: Constant::Integer(2),
                },
            ],
        };
        let control = parse_quote! { [1i64, 2i64] };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_block_from_lir_block() {
        let expression = Expression::Block {
            block: Block {
                statements: vec![
                    Statement::Let {
                        identifier: String::from("x"),
                        expression: Expression::Literal {
                            literal: Constant::Integer(1),
                        },
                    },
                    Statement::ExpressionLast {
                        expression: Expression::Identifier {
                            identifier: String::from("x"),
                        },
                    },
                ],
            },
        };
        let control = parse_quote! { { let x = 1i64; x } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_function_call_from_lir_function_call() {
        let expression = Expression::FunctionCall {
            function: Box::new(Expression::Identifier {
                identifier: String::from("foo"),
            }),
            arguments: vec![
                Expression::Identifier {
                    identifier: String::from("a"),
                },
                Expression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };

        let control = parse_quote! { foo (a, b) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_binary_from_lir_function_call() {
        let expression = Expression::FunctionCall {
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
        };

        let control = parse_quote! { a + b };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_method_call_from_lir_node_call() {
        let expression = Expression::NodeCall {
            node_identifier: String::from("node_state"),
            input_name: String::from("NodeInput"),
            input_fields: vec![(
                String::from("i"),
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
            )],
        };

        let control = parse_quote! { self . node_state . step ( NodeInput { i : 1i64 }) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_field_access_from_lir_field_access() {
        let expression = Expression::FieldAccess {
            expression: Box::new(Expression::Identifier {
                identifier: String::from("my_point"),
            }),
            field: FieldIdentifier::Named(String::from("x")),
        };

        let control = parse_quote! { my_point . x };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_closure_from_lir_lambda() {
        let expression = Expression::Lambda {
            inputs: vec![(String::from("x"), Type::Integer)],
            output: Type::Integer,
            body: Box::new(Expression::Block {
                block: Block {
                    statements: vec![
                        Statement::Let {
                            identifier: String::from("y"),
                            expression: Expression::Identifier {
                                identifier: String::from("x"),
                            },
                        },
                        Statement::ExpressionLast {
                            expression: Expression::Identifier {
                                identifier: String::from("y"),
                            },
                        },
                    ],
                },
            }),
        };

        let control = parse_quote! { move |x: i64| -> i64 { let y = x; y } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_ifthenelse_from_lir_ifthenelse() {
        let expression = Expression::IfThenElse {
            condition: Box::new(Expression::Identifier {
                identifier: String::from("test"),
            }),
            then_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                }],
            },
            else_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Literal {
                        literal: Constant::Integer(0),
                    },
                }],
            },
        };

        let control = parse_quote! { if test { 1i64 } else { 0i64 } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_match_from_lir_match() {
        let expression = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: String::from("my_color"),
            }),
            arms: vec![
                (
                    Pattern::Enumeration {
                        enum_name: String::from("Color"),
                        elem_name: format!("Blue"),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
                (
                    Pattern::Enumeration {
                        enum_name: String::from("Color"),
                        elem_name: format!("Green"),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(0),
                    },
                ),
            ],
        };

        let control =
            parse_quote! { match my_color { Color::Blue => 1i64, Color::Green => 0i64, } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_map_operation_from_lir_map() {
        let expression = Expression::Map {
            mapped: Box::new(Expression::Identifier {
                identifier: format!("a"),
            }),
            function: Box::new(Expression::Identifier {
                identifier: format!("f"),
            }),
        };

        let control = parse_quote! { a . map (f) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_fold_iterator_from_lir_fold() {
        let expression = Expression::Fold {
            folded: Box::new(Expression::Identifier {
                identifier: format!("a"),
            }),
            initialization: Box::new(Expression::Literal {
                literal: Constant::Integer(0),
            }),
            function: Box::new(Expression::Identifier {
                identifier: format!("sum"),
            }),
        };

        let control = parse_quote! { a . into_iter().fold(0i64, sum) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_sort_iterator_from_lir_sort() {
        let expression = Expression::Sort {
            sorted: Box::new(Expression::Identifier {
                identifier: format!("a"),
            }),
            function: Box::new(Expression::Identifier {
                identifier: format!("compare"),
            }),
        };

        let control = parse_quote! { a . sort (compare) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_macro_from_lir_zip() {
        let expression = Expression::Zip {
            arrays: vec![
                Expression::Identifier {
                    identifier: format!("a"),
                },
                Expression::Identifier {
                    identifier: format!("b"),
                },
            ],
        };

        let control = parse_quote! { par_zip!(a, b) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }
}
