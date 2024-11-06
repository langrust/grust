//! LIR [Expr] module.

prelude! {
    operator::{BinaryOperator, UnaryOperator},
    Pattern, Block,
}

/// LIR expressions.
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: String,
    },
    /// Some expression: `Some(x`.
    Some {
        /// The expression.
        expression: Box<Self>,
    },
    /// None value: `None`.
    None,
    /// An unitary operation: `!x`.
    Unop {
        /// The operator.
        op: UnaryOperator,
        /// The expression.
        expression: Box<Self>,
    },
    /// A binary operation: `x + y`.
    Binop {
        /// The operator.
        op: BinaryOperator,
        /// The left expression.
        left_expression: Box<Self>,
        /// The right expression.
        right_expression: Box<Self>,
    },
    /// An if_then_else expression: `if test { "ok" } else { "oh no" }`.
    IfThenElse {
        /// The test expression.
        condition: Box<Self>,
        /// The `true` block.
        then_branch: Block,
        /// The `false` block.
        else_branch: Block,
    },
    /// A memory access: `self.i_mem`.
    MemoryAccess {
        /// The identifier to the memory.
        identifier: String,
    },
    /// An input access: `self.i_mem`.
    InputAccess {
        /// The identifier to the input.
        identifier: String,
    },
    /// A structure literal expression: `Point { x: 1, y: 1 }`.
    Structure {
        /// The name of the structure.
        name: String,
        /// The filled fields.
        fields: Vec<(String, Self)>,
    },
    /// A enumeration literal expression: `Color::Red`.
    Enumeration {
        /// The name of the enumeration.
        name: String,
        /// The name of the element.
        element: String,
    },
    /// An array expression: `[1, 2, 3]`.
    Array {
        /// The elements inside the array.
        elements: Vec<Self>,
    },
    /// A tuple expression: `(1, 2, 3)`.
    Tuple {
        /// The elements inside the tuple.
        elements: Vec<Self>,
    },
    /// A block scope: `{ let x = 1; x }`.
    Block {
        /// The block.
        block: Block,
    },
    /// A function call: `foo(x, y)`.
    FunctionCall {
        /// The function called.
        function: Box<Self>,
        /// The arguments.
        arguments: Vec<Self>,
    },
    /// A node call: `self.called_node.step(inputs)`.
    NodeCall {
        /// Node's identifier in memory.
        memory_ident: String,
        /// The identifier to the node.
        node_identifier: String,
        /// The name of the input structure of the called node.
        input_name: String,
        /// The filled input's fields.
        input_fields: Vec<(String, Self)>,
    },
    /// A named or unamed field access: `my_point.x`.
    FieldAccess {
        /// The structure or tuple typed expression.
        expression: Box<Self>,
        /// The identifier of the field.
        field: FieldIdentifier,
    },
    /// A lambda expression: `|x, y| x * y`.
    Lambda {
        /// The lambda inputs.
        inputs: Vec<(String, Typ)>,
        /// The output type.
        output: Typ,
        /// The body of the closure.
        body: Box<Self>,
    },
    /// A match expression: `match c { Color::Blue => 1, _ => 0, }`
    Match {
        /// The matched expression.
        matched: Box<Self>,
        /// The pattern matching arms.
        arms: Vec<(Pattern, Option<Self>, Self)>,
    },
    /// A map expression: `my_list.map(|x| x + 1)`
    Map {
        /// The mapped expression.
        mapped: Box<Self>,
        /// The mapping function.
        function: Box<Self>,
    },
    /// A fold expression: `my_list.fold(0, |sum, x| x + sum)`
    Fold {
        /// The folded expression.
        folded: Box<Self>,
        /// The initialization expression.
        initialization: Box<Self>,
        /// The folding function.
        function: Box<Self>,
    },
    /// A sort expression: `my_list.map(|a, b| a - b)`
    Sort {
        /// The sorted expression.
        sorted: Box<Self>,
        /// The sorting function.
        function: Box<Self>,
    },
    /// Arrays zip operator expression: `zip(a, b, [1, 2, 3])`
    Zip {
        /// The arrays expression.
        arrays: Vec<Self>,
    },
}

impl Expr {
    mk_new! {
        Literal: literal { literal: Constant }
        Literal: lit { literal: Constant }
        Identifier: ident { identifier: impl Into<String> = identifier.into() }
        Some: some {
            expression: Self = Box::new(expression),
        }
        None: none ()
        Unop: unop {
            op: UnaryOperator,
            expression: Self = Box::new(expression),
        }
        Binop: binop {
            op: BinaryOperator,
            left_expression: Self = left_expression.into(),
            right_expression: Self = right_expression.into(),
        }
        IfThenElse: ite {
            condition: Self = Box::new(condition),
            then_branch: Block,
            else_branch: Block,
        }
        MemoryAccess: memory_access { identifier: impl Into<String> = identifier.into() }
        InputAccess: input_access { identifier: impl Into<String> = identifier.into() }
        Structure: structure {
            name: impl Into<String> = name.into(),
            fields: Vec<(String, Self)>
        }
        Enumeration: enumeration {
            name: impl Into<String> = name.into(),
            element: impl Into<String> = element.into(),
        }
        Array: array { elements: Vec<Self> }
        Tuple: tuple { elements: Vec<Self> }
        Block: block { block: Block }
        FunctionCall: function_call {
            function: Self = function.into(),
            arguments: Vec<Self>,
        }
        NodeCall: node_call {
            memory_ident: impl Into<String> = memory_ident.into(),
            node_identifier: impl Into<String> = node_identifier.into(),
            input_name: impl Into<String> = input_name.into(),
            input_fields: Vec<(String, Self)>,
        }
        FieldAccess: field_access {
            expression: Self = expression.into(),
            field: FieldIdentifier
        }
        Lambda: lambda {
            inputs: Vec<(String, Typ)>,
            output: Typ,
            body: Self = body.into(),
        }
        Match: pat_match {
            matched: Self = matched.into(),
            arms: Vec<(Pattern, Option<Self>, Self)>
        }
        Map: map {
            mapped: Self = mapped.into(),
            function: Self = function.into(),
        }
        Fold: fold {
            folded: Self = folded.into(),
            initialization: Self = initialization.into(),
            function: Self = function.into(),
        }
        Sort: sort {
            sorted: Self = sorted.into(),
            function: Self = function.into()
        }
        Zip: zip { arrays: Vec<Self> }
    }

    /// True on expressions that require parens to be used as a function in a function call.
    ///
    /// More precisely assume a call like `<expr>(<params>)`, then this function returns `true` iff
    /// `<expr>` should be wrapped in parens for the whole call to be legal rust.
    pub fn as_function_requires_parens(&self) -> bool {
        use Expr::*;
        match self {
            Literal { .. }
            | Identifier { .. }
            | Some { .. }
            | None { .. }
            | MemoryAccess { .. }
            | InputAccess { .. }
            | Enumeration { .. }
            | Array { .. }
            | Tuple { .. }
            | Block { .. }
            | FieldAccess { .. } => false,
            Unop { .. }
            | Binop { .. }
            | IfThenElse { .. }
            | Structure { .. }
            | FunctionCall { .. }
            | NodeCall { .. }
            | Lambda { .. }
            | Match { .. }
            | Map { .. }
            | Fold { .. }
            | Sort { .. }
            | Zip { .. } => true,
        }
    }

    /// True on expressions that require parens to be used as a argument to unary or binary operations.
    pub fn as_op_arg_requires_parens(&self) -> bool {
        use Expr::*;
        match self {
            Binop { .. } | IfThenElse { .. } | Lambda { .. } => true,
            Literal { .. }
            | Identifier { .. }
            | Unop { .. }
            | Some { .. }
            | None { .. }
            | FunctionCall { .. }
            | NodeCall { .. }
            | MemoryAccess { .. }
            | InputAccess { .. }
            | FieldAccess { .. }
            | Enumeration { .. }
            | Structure { .. }
            | Array { .. }
            | Tuple { .. }
            | Block { .. }
            | Match { .. }
            | Map { .. }
            | Fold { .. }
            | Sort { .. }
            | Zip { .. } => false,
        }
    }
    /// Transform LIR expression into RustAST expression.
    pub fn to_syn(self, crates: &mut BTreeSet<String>) -> syn::Expr {
        match self {
            Self::Literal { literal } => literal.to_syn(),
            Self::Identifier { identifier } => {
                let identifier = Ident::new(&identifier, Span::call_site());
                parse_quote! { #identifier }
            }
            Self::Some { expression } => {
                let syn_expr = expression.to_syn(crates);
                parse_quote! { Some(#syn_expr) }
            }
            Self::None => parse_quote! { None },
            Self::MemoryAccess { identifier } => {
                let id = format_ident!("last_{}", identifier);
                parse_quote!( self.#id )
            }
            Self::InputAccess { identifier } => {
                let identifier = format_ident!("{identifier}");
                parse_quote!( input.#identifier )
            }
            Self::Structure { name, fields } => {
                let fields: Vec<syn::FieldValue> = fields
                    .into_iter()
                    .map(|(name, expression)| {
                        let name = format_ident!("{name}");
                        let expression = expression.to_syn(crates);
                        parse_quote!(#name : #expression)
                    })
                    .collect();
                let name = format_ident!("{name}");
                parse_quote!(#name { #(#fields),* })
            }
            Self::Enumeration { name, element } => {
                syn::parse_str(&format!("{name}::{element}")).unwrap()
            }
            Self::Array { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|expression| expression.to_syn(crates));
                parse_quote! { [#(#elements),*]}
            }
            Self::Tuple { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|expression| expression.to_syn(crates));
                parse_quote! { (#(#elements),*) }
            }
            Self::Block { block } => syn::Expr::Block(syn::ExprBlock {
                attrs: vec![],
                label: None,
                block: block.to_syn(crates),
            }),
            Self::FunctionCall {
                function,
                arguments,
            } => {
                let function_parens = function.as_function_requires_parens();
                let function = function.to_syn(crates);
                let arguments = arguments
                    .into_iter()
                    .map(|expression| expression.to_syn(crates));
                if function_parens {
                    parse_quote! { (#function)(#(#arguments),*) }
                } else {
                    parse_quote! { #function(#(#arguments),*) }
                }
            }
            Self::Unop { op, expression } => {
                let op = op.to_syn();
                let expr = expression.to_syn(crates);
                syn::Expr::Unary(parse_quote! { #op (#expr) })
            }
            Self::Binop {
                op,
                left_expression,
                right_expression,
            } => {
                let left = if left_expression.as_op_arg_requires_parens() {
                    let expr = left_expression.to_syn(crates);
                    parse_quote! { (#expr) }
                } else {
                    left_expression.to_syn(crates)
                };
                let right = if right_expression.as_op_arg_requires_parens() {
                    let expr = right_expression.to_syn(crates);
                    parse_quote! { (#expr) }
                } else {
                    right_expression.to_syn(crates)
                };
                let binary = op.to_syn();
                syn::Expr::Binary(parse_quote! { #left #binary #right })
            }
            Self::NodeCall {
                memory_ident,
                input_name,
                input_fields,
                ..
            } => {
                let ident = Ident::new(&memory_ident, Span::call_site());
                let receiver: syn::ExprField = parse_quote! { self.#ident};
                let input_fields: Vec<syn::FieldValue> = input_fields
                    .into_iter()
                    .map(|(name, expression)| {
                        let id = Ident::new(&name, Span::call_site());
                        let expr = expression.to_syn(crates);
                        parse_quote! { #id : #expr }
                    })
                    .collect();

                let input_name = Ident::new(&input_name, Span::call_site());
                let argument: syn::ExprStruct = parse_quote! { #input_name { #(#input_fields),* }};

                syn::Expr::MethodCall(parse_quote! { #receiver.step (#argument) })
            }
            Self::FieldAccess { expression, field } => {
                let expression = expression.to_syn(crates);
                match field {
                    FieldIdentifier::Named(name) => {
                        let name = Ident::new(&name, Span::call_site());
                        parse_quote!(#expression.#name)
                    }
                    FieldIdentifier::Unamed(number) => {
                        let number: TokenStream2 = format!("{number}").parse().unwrap();
                        parse_quote!(#expression.#number)
                    }
                }
            }
            Self::Lambda {
                inputs,
                output,
                body,
            } => {
                let inputs = inputs
                    .into_iter()
                    .map(|(identifier, typ)| {
                        let pattern = syn::Pat::Ident(syn::PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new(&identifier, Span::call_site()),
                            subpat: None,
                        });
                        let pattern = syn::Pat::Type(syn::PatType {
                            attrs: Vec::new(),
                            pat: Box::new(pattern),
                            colon_token: Default::default(),
                            ty: Box::new(typ.to_syn()),
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
                    output: syn::ReturnType::Type(Default::default(), Box::new(output.to_syn())),
                    body: Box::new(body.to_syn(crates)),
                    lifetimes: None,
                    constness: None,
                };
                syn::Expr::Closure(closure)
            }
            Self::IfThenElse {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = Box::new(condition.to_syn(crates));
                let then_branch = then_branch.to_syn(crates);
                let else_branch = else_branch.to_syn(crates);
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
            Self::Match { matched, arms } => {
                let arms = arms
                    .into_iter()
                    .map(|(pattern, guard, body)| syn::Arm {
                        attrs: Vec::new(),
                        pat: pattern.to_syn(),
                        guard: guard
                            .map(|expression| expression.to_syn(crates))
                            .map(|g| (Default::default(), Box::new(g))),
                        body: Box::new(body.to_syn(crates)),
                        fat_arrow_token: Default::default(),
                        comma: Some(Default::default()),
                    })
                    .collect();
                syn::Expr::Match(syn::ExprMatch {
                    attrs: Vec::new(),
                    match_token: Default::default(),
                    expr: Box::new(matched.to_syn(crates)),
                    brace_token: Default::default(),
                    arms,
                })
            }
            Self::Map { mapped, function } => {
                let receiver = Box::new(mapped.to_syn(crates));
                let method = Ident::new("map", Span::call_site());
                let arguments = vec![function.to_syn(crates)];
                let method_call = syn::ExprMethodCall {
                    attrs: Vec::new(),
                    receiver,
                    method,
                    turbofish: None,
                    paren_token: Default::default(),
                    args: syn::Punctuated::from_iter(arguments),
                    dot_token: Default::default(),
                };
                syn::Expr::MethodCall(method_call)
            }
            Self::Fold {
                folded,
                initialization,
                function,
            } => syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs: Vec::new(),
                receiver: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                    attrs: Vec::new(),
                    receiver: Box::new(folded.to_syn(crates)),
                    method: Ident::new("into_iter", Span::call_site()),
                    turbofish: None,
                    paren_token: Default::default(),
                    args: syn::Punctuated::new(),
                    dot_token: Default::default(),
                })),
                method: Ident::new("fold", Span::call_site()),
                turbofish: None,
                paren_token: Default::default(),
                args: syn::Punctuated::from_iter(vec![
                    initialization.to_syn(crates),
                    function.to_syn(crates),
                ]),
                dot_token: Default::default(),
            }),
            Self::Sort { sorted, function } => {
                let token_sorted = sorted.to_syn(crates);
                let token_function = function.to_syn(crates);

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
            Self::Zip { arrays } => {
                crates.insert("itertools = \"0.12.1\"".into());
                let arguments = arrays
                    .into_iter()
                    .map(|expression| expression.to_syn(crates));
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
}

/// LIR field access member.
#[derive(Debug, PartialEq)]
pub enum FieldIdentifier {
    /// Named field access.
    Named(String),
    /// Unamed field access.
    Unamed(usize),
}

impl FieldIdentifier {
    pub fn named(s: impl Into<String>) -> Self {
        Self::Named(s.into())
    }
    pub fn unamed(n: usize) -> Self {
        Self::Unamed(n)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_literal_from_lir_literal() {
        let expression = Expr::lit(Constant::Integer(parse_quote!(1i64)));
        let control = parse_quote! { 1i64 };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_identifier_from_lir_identifier() {
        let expression = Expr::ident("x");
        let control = parse_quote! { x };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_self_from_lir_memory_access() {
        let expression = Expr::memory_access("x");
        let control = parse_quote! { self.last_x};
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_input_from_lir_input_access() {
        let expression = Expr::input_access("i");
        let control = parse_quote! { input.i};
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_structure_from_lir_structure() {
        let expression = Expr::structure(
            "Point",
            vec![
                ("x".into(), Expr::lit(Constant::Integer(parse_quote!(1i64)))),
                ("y".into(), Expr::lit(Constant::Integer(parse_quote!(2i64)))),
            ],
        );
        let control = parse_quote! { Point { x : 1i64, y : 2i64 } };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_array_from_lir_array() {
        let expression = Expr::array(vec![
            Expr::lit(Constant::Integer(parse_quote!(1i64))),
            Expr::lit(Constant::Integer(parse_quote!(2i64))),
        ]);
        let control = parse_quote! { [1i64, 2i64] };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_block_from_lir_block() {
        let expression = Expr::block(Block {
            statements: vec![
                Stmt::Let {
                    pattern: Pattern::ident("x"),
                    expression: Expr::lit(Constant::Integer(parse_quote!(1i64))),
                },
                Stmt::ExprLast {
                    expression: Expr::ident("x"),
                },
            ],
        });
        let control = parse_quote! { { let x = 1i64; x } };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_function_call_from_lir_function_call() {
        let expression =
            Expr::function_call(Expr::ident("foo"), vec![Expr::ident("a"), Expr::ident("b")]);

        let control = parse_quote! { foo (a, b) };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_binary_from_lir_function_call() {
        let expression = Expr::binop(
            operator::BinaryOperator::Add,
            Expr::ident("a"),
            Expr::ident("b"),
        );

        let control = parse_quote! { a + b };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_method_call_from_lir_node_call() {
        let expression = Expr::node_call(
            "node_state",
            "node",
            "NodeInput",
            vec![(
                "i".into(),
                Expr::Literal {
                    literal: Constant::Integer(parse_quote!(1i64)),
                },
            )],
        );

        let control = parse_quote! { self.node_state.step ( NodeInput { i : 1i64 }) };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_from_lir_field_access() {
        let expression =
            Expr::field_access(Expr::ident("my_point"), FieldIdentifier::Named("x".into()));

        let control = parse_quote! { my_point.x };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_closure_from_lir_lambda() {
        let expression = Expr::lambda(
            vec![("x".into(), Typ::int())],
            Typ::int(),
            Expr::block(Block {
                statements: vec![
                    Stmt::let_binding(Pattern::ident("y"), Expr::ident("x")),
                    Stmt::expression_last(Expr::ident("y")),
                ],
            }),
        );

        let control = parse_quote! { move |x: i64| -> i64 { let y = x; y } };
        assert_eq!(expression.to_syn(&mut Default::default()), control,)
    }

    #[test]
    fn should_create_rust_ast_ifthenelse_from_lir_ifthenelse() {
        let expression = Expr::ite(
            Expr::ident("test"),
            Block::new(vec![Stmt::expression_last(Expr::lit(Constant::int(
                parse_quote!(1i64),
            )))]),
            Block::new(vec![Stmt::expression_last(Expr::lit(Constant::int(
                parse_quote!(0i64),
            )))]),
        );

        let control = parse_quote! { if test { 1i64 } else { 0i64 } };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_match_from_lir_match() {
        let expression = Expr::pat_match(
            Expr::ident("my_color"),
            vec![
                (
                    Pattern::enumeration("Color", "Blue", None),
                    None,
                    Expr::lit(Constant::Integer(parse_quote!(1i64))),
                ),
                (
                    Pattern::enumeration("Color", "Green", None),
                    None,
                    Expr::Literal {
                        literal: Constant::Integer(parse_quote!(0i64)),
                    },
                ),
            ],
        );

        let control =
            parse_quote! { match my_color { Color::Blue => 1i64, Color::Green => 0i64, } };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_map_operation_from_lir_map() {
        let expression = Expr::map(Expr::ident("a"), Expr::ident("f"));

        let control = parse_quote! { a.map (f) };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_fold_iterator_from_lir_fold() {
        let expression = Expr::fold(
            Expr::ident("a"),
            Expr::lit(Constant::Integer(parse_quote!(0i64))),
            Expr::ident("sum"),
        );

        let control = parse_quote! { a.into_iter().fold(0i64, sum) };
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_sort_iterator_from_lir_sort() {
        let expression = Expr::sort(Expr::ident("a"), Expr::ident("compare"));

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
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_macro_from_lir_zip() {
        let expression = Expr::zip(vec![Expr::ident("a"), Expr::ident("b")]);

        let control = parse_quote!({
            let mut iter = itertools::izip!(a, b);
            std::array::from_fn(|_| iter.next().unwrap())
        });
        assert_eq!(expression.to_syn(&mut Default::default()), control)
    }
}
