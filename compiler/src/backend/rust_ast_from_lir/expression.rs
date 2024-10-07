use std::collections::BTreeSet;

prelude! {
    macro2::{Span, TokenStream},
    quote::format_ident,
    syn::*,
    operator::*,
    lir::FieldIdentifier,
}

use super::{
    block::rust_ast_from_lir as block_rust_ast_from_lir,
    pattern::rust_ast_from_lir as pattern_rust_ast_from_lir,
    r#type::rust_ast_from_lir as type_rust_ast_from_lir,
};

/// Transforms binary operator into syn's binary operator.
pub fn binary_to_syn(op: BinaryOperator) -> BinOp {
    match op {
        BinaryOperator::Mul => BinOp::Mul(Default::default()),
        BinaryOperator::Div => BinOp::Div(Default::default()),
        BinaryOperator::Mod => BinOp::Rem(Default::default()),
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
pub fn unary_to_syn(op: UnaryOperator) -> syn::UnOp {
    match op {
        UnaryOperator::Neg => UnOp::Neg(Default::default()),
        UnaryOperator::Not => UnOp::Not(Default::default()),
    }
}

/// Transforms constants into syn constants.
pub fn constant_to_syn(literal: Constant) -> Expr {
    match literal {
        Constant::Integer(i) => Expr::Lit(ExprLit {
            attrs: vec![],
            lit: syn::Lit::Int(LitInt::new(
                &(i.base10_digits().to_owned() + "i64"),
                i.span(),
            )),
        }),
        Constant::Float(f) => Expr::Lit(ExprLit {
            attrs: vec![],
            lit: {
                // force `f64` suffix
                let f = if f.suffix() == "" {
                    let mut s = f.to_string();
                    // careful on trailing `.`
                    if s.ends_with(".") {
                        s.push('0');
                    }
                    s.push_str("f64");
                    syn::LitFloat::new(&s, f.span())
                } else {
                    f
                };
                syn::Lit::Float(f)
            },
        }),
        Constant::Boolean(b) => Expr::Lit(ExprLit {
            attrs: vec![],
            lit: syn::Lit::Bool(b),
        }),
        Constant::Unit(paren_token) => Expr::Tuple(ExprTuple {
            attrs: vec![],
            paren_token,
            elems: Default::default(),
        }),
        Constant::Default => parse_quote! { Default::default() },
    }
}

/// Transform LIR expression into RustAST expression.
pub fn rust_ast_from_lir(expression: lir::Expr, crates: &mut BTreeSet<String>) -> Expr {
    match expression {
        lir::Expr::Literal { literal } => constant_to_syn(literal),
        lir::Expr::Identifier { identifier } => {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote! { #identifier }
        }
        lir::Expr::Some { expression } => {
            let syn_expr = rust_ast_from_lir(*expression, crates);
            parse_quote! { Some(#syn_expr) }
        }
        lir::Expr::None => parse_quote! { None },
        lir::Expr::MemoryAccess { identifier } => {
            let id = format_ident!("last_{}", identifier);
            parse_quote!( self.#id )
        }
        lir::Expr::InputAccess { identifier } => {
            let identifier = format_ident!("{identifier}");
            parse_quote!( input.#identifier )
        }
        lir::Expr::Structure { name, fields } => {
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
        lir::Expr::Enumeration { name, element } => {
            syn::parse_str(&format!("{name}::{element}")).unwrap()
        }
        lir::Expr::Array { elements } => {
            let elements = elements
                .into_iter()
                .map(|expression| rust_ast_from_lir(expression, crates));
            parse_quote! { [#(#elements),*]}
        }
        lir::Expr::Tuple { elements } => {
            let elements = elements
                .into_iter()
                .map(|expression| rust_ast_from_lir(expression, crates));
            parse_quote! { (#(#elements),*) }
        }
        lir::Expr::Block { block } => Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: block_rust_ast_from_lir(block, crates),
        }),
        lir::Expr::FunctionCall {
            function,
            arguments,
        } => {
            let function_parens = function.as_function_requires_parens();
            let function = rust_ast_from_lir(*function, crates);
            let arguments = arguments
                .into_iter()
                .map(|expression| rust_ast_from_lir(expression, crates));
            if function_parens {
                parse_quote! { (#function)(#(#arguments),*) }
            } else {
                parse_quote! { #function(#(#arguments),*) }
            }
        }
        lir::Expr::Unop { op, expression } => {
            let op = unary_to_syn(op);
            let expr = rust_ast_from_lir(*expression, crates);
            Expr::Unary(parse_quote! { #op (#expr) })
        }
        lir::Expr::Binop {
            op,
            left_expression,
            right_expression,
        } => {
            let left = if left_expression.as_op_arg_requires_parens() {
                let expr = rust_ast_from_lir(*left_expression, crates);
                parse_quote! { (#expr) }
            } else {
                rust_ast_from_lir(*left_expression, crates)
            };
            let right = if right_expression.as_op_arg_requires_parens() {
                let expr = rust_ast_from_lir(*right_expression, crates);
                parse_quote! { (#expr) }
            } else {
                rust_ast_from_lir(*right_expression, crates)
            };
            let binary = binary_to_syn(op);
            Expr::Binary(parse_quote! { #left #binary #right })
        }
        lir::Expr::NodeCall {
            memory_ident,
            input_name,
            input_fields,
            ..
        } => {
            let ident = Ident::new(&memory_ident, Span::call_site());
            let receiver: ExprField = parse_quote! { self.#ident};
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

            Expr::MethodCall(parse_quote! { #receiver.step (#argument) })
        }
        lir::Expr::FieldAccess { expression, field } => {
            let expression = rust_ast_from_lir(*expression, crates);
            match field {
                FieldIdentifier::Named(name) => {
                    let name = Ident::new(&name, Span::call_site());
                    parse_quote!(#expression.#name)
                }
                FieldIdentifier::Unamed(number) => {
                    let number: TokenStream = format!("{number}").parse().unwrap();
                    parse_quote!(#expression.#number)
                }
            }
        }
        lir::Expr::Lambda {
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
                        ident: syn::Ident::new(&identifier, Span::call_site()),
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
                    span: Span::call_site(),
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
        lir::Expr::IfThenElse {
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
        lir::Expr::Match { matched, arms } => {
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
        lir::Expr::Map { mapped, function } => {
            let receiver = Box::new(rust_ast_from_lir(*mapped, crates));
            let method = syn::Ident::new("map", Span::call_site());
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
        lir::Expr::Fold {
            folded,
            initialization,
            function,
        } => syn::Expr::MethodCall(syn::ExprMethodCall {
            attrs: Vec::new(),
            receiver: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs: Vec::new(),
                receiver: Box::new(rust_ast_from_lir(*folded, crates)),
                method: syn::Ident::new("into_iter", Span::call_site()),
                turbofish: None,
                paren_token: Default::default(),
                args: syn::punctuated::Punctuated::new(),
                dot_token: Default::default(),
            })),
            method: syn::Ident::new("fold", Span::call_site()),
            turbofish: None,
            paren_token: Default::default(),
            args: syn::punctuated::Punctuated::from_iter(vec![
                rust_ast_from_lir(*initialization, crates),
                rust_ast_from_lir(*function, crates),
            ]),
            dot_token: Default::default(),
        }),
        lir::Expr::Sort { sorted, function } => {
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
        lir::Expr::Zip { arrays } => {
            crates.insert("itertools = \"0.12.1\"".into());
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
    prelude! {
        backend::rust_ast_from_lir::expression::rust_ast_from_lir,
        lir::{ Block, FieldIdentifier, Pattern, Stmt },
        syn::parse_quote,
    }

    #[test]
    fn should_create_rust_ast_literal_from_lir_literal() {
        let expression = lir::Expr::lit(Constant::Integer(parse_quote!(1i64)));
        let control = parse_quote! { 1i64 };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_identifier_from_lir_identifier() {
        let expression = lir::Expr::ident("x");
        let control = parse_quote! { x };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_field_access_to_self_from_lir_memory_access() {
        let expression = lir::Expr::memory_access("x");
        let control = parse_quote! { self.last_x};
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_field_access_to_input_from_lir_input_access() {
        let expression = lir::Expr::input_access("i");
        let control = parse_quote! { input.i};
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_structure_from_lir_structure() {
        let expression = lir::Expr::structure(
            "Point",
            vec![
                (
                    "x".into(),
                    lir::Expr::lit(Constant::Integer(parse_quote!(1i64))),
                ),
                (
                    "y".into(),
                    lir::Expr::lit(Constant::Integer(parse_quote!(2i64))),
                ),
            ],
        );
        let control = parse_quote! { Point { x : 1i64, y : 2i64 } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_array_from_lir_array() {
        let expression = lir::Expr::array(vec![
            lir::Expr::lit(Constant::Integer(parse_quote!(1i64))),
            lir::Expr::lit(Constant::Integer(parse_quote!(2i64))),
        ]);
        let control = parse_quote! { [1i64, 2i64] };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_block_from_lir_block() {
        let expression = lir::Expr::block(Block {
            statements: vec![
                Stmt::Let {
                    pattern: Pattern::ident("x"),
                    expression: lir::Expr::lit(Constant::Integer(parse_quote!(1i64))),
                },
                Stmt::ExprLast {
                    expression: lir::Expr::ident("x"),
                },
            ],
        });
        let control = parse_quote! { { let x = 1i64; x } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_function_call_from_lir_function_call() {
        let expression = lir::Expr::function_call(
            lir::Expr::ident("foo"),
            vec![lir::Expr::ident("a"), lir::Expr::ident("b")],
        );

        let control = parse_quote! { foo (a, b) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_binary_from_lir_function_call() {
        let expression = lir::Expr::binop(
            super::BinaryOperator::Add,
            lir::Expr::ident("a"),
            lir::Expr::ident("b"),
        );

        let control = parse_quote! { a + b };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_method_call_from_lir_node_call() {
        let expression = lir::Expr::node_call(
            "node_state",
            "node",
            "NodeInput",
            vec![(
                "i".into(),
                lir::Expr::Literal {
                    literal: Constant::Integer(parse_quote!(1i64)),
                },
            )],
        );

        let control = parse_quote! { self.node_state.step ( NodeInput { i : 1i64 }) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_field_access_from_lir_field_access() {
        let expression = lir::Expr::field_access(
            lir::Expr::ident("my_point"),
            FieldIdentifier::Named("x".into()),
        );

        let control = parse_quote! { my_point.x };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_closure_from_lir_lambda() {
        let expression = lir::Expr::lambda(
            vec![("x".into(), Typ::int())],
            Typ::int(),
            lir::Expr::block(Block {
                statements: vec![
                    Stmt::let_binding(Pattern::ident("y"), lir::Expr::ident("x")),
                    Stmt::expression_last(lir::Expr::ident("y")),
                ],
            }),
        );

        let control = parse_quote! { move |x: i64| -> i64 { let y = x; y } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control,
        )
    }

    #[test]
    fn should_create_rust_ast_ifthenelse_from_lir_ifthenelse() {
        let expression = lir::Expr::ite(
            lir::Expr::ident("test"),
            Block::new(vec![Stmt::expression_last(lir::Expr::lit(Constant::int(
                parse_quote!(1i64),
            )))]),
            Block::new(vec![Stmt::expression_last(lir::Expr::lit(Constant::int(
                parse_quote!(0i64),
            )))]),
        );

        let control = parse_quote! { if test { 1i64 } else { 0i64 } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_match_from_lir_match() {
        let expression = lir::Expr::pat_match(
            lir::Expr::ident("my_color"),
            vec![
                (
                    Pattern::enumeration("Color", "Blue", None),
                    None,
                    lir::Expr::lit(Constant::Integer(parse_quote!(1i64))),
                ),
                (
                    Pattern::enumeration("Color", "Green", None),
                    None,
                    lir::Expr::Literal {
                        literal: Constant::Integer(parse_quote!(0i64)),
                    },
                ),
            ],
        );

        let control =
            parse_quote! { match my_color { Color::Blue => 1i64, Color::Green => 0i64, } };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_map_operation_from_lir_map() {
        let expression = lir::Expr::map(lir::Expr::ident("a"), lir::Expr::ident("f"));

        let control = parse_quote! { a.map (f) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_fold_iterator_from_lir_fold() {
        let expression = lir::Expr::fold(
            lir::Expr::ident("a"),
            lir::Expr::lit(Constant::Integer(parse_quote!(0i64))),
            lir::Expr::ident("sum"),
        );

        let control = parse_quote! { a.into_iter().fold(0i64, sum) };
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_sort_iterator_from_lir_sort() {
        let expression = lir::Expr::sort(lir::Expr::ident("a"), lir::Expr::ident("compare"));

        let control = parse_quote!({
            let mut x = a.clone();
            let slice = x.as_mut();
            slice.sort_by(|a, b| {
                let compare = compare(*a, *b);
                if compare < 0 {
                    std::cmp::Ordering::Less
                } else if compare > 0 {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            x
        });
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_macro_from_lir_zip() {
        let expression = lir::Expr::zip(vec![lir::Expr::ident("a"), lir::Expr::ident("b")]);

        let control = parse_quote!({
            let mut iter = itertools::izip!(a, b);
            std::array::from_fn(|_| iter.next().unwrap())
        });
        assert_eq!(
            rust_ast_from_lir(expression, &mut Default::default()),
            control
        )
    }
}
