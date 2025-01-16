//! [Expr] module.

prelude! {}

/// Expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: Ident,
    },
    /// Some expression: `Some(x`.
    Some {
        /// The expression.
        expr: Box<Self>,
    },
    /// None value: `None`.
    None,
    /// An unitary operation: `!x`.
    UnOp {
        /// The operator.
        op: UOp,
        /// The expression.
        expr: Box<Self>,
    },
    /// A binary operation: `x + y`.
    BinOp {
        /// The operator.
        op: BOp,
        /// The left expression.
        lft: Box<Self>,
        /// The right expression.
        rgt: Box<Self>,
    },
    /// An if_then_else expression: `if test { "ok" } else { "oh no" }`.
    IfThenElse {
        /// Condition.
        cnd: Box<Self>,
        /// `then` branch.
        thn: Block,
        /// `else` branch.
        els: Block,
    },
    /// A memory access: `self.i_mem`.
    MemoryAccess {
        /// The identifier to the memory.
        identifier: Ident,
    },
    /// An input access: `self.i_mem`.
    InputAccess {
        /// The identifier to the input.
        identifier: Ident,
    },
    /// A structure literal expression: `Point { x: 1, y: 1 }`.
    Structure {
        /// The name of the structure.
        name: Ident,
        /// The filled fields.
        fields: Vec<(Ident, Self)>,
    },
    /// A enumeration literal expression: `Color::Red`.
    Enumeration {
        /// The name of the enumeration.
        name: Ident,
        /// The name of the element.
        element: Ident,
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
        memory_ident: Ident,
        /// The identifier to the node.
        node_identifier: Ident,
        /// The name of the input structure of the called node.
        input_name: Ident,
        /// The filled input's fields.
        input_fields: Vec<(Ident, Self)>,
    },
    /// A named or unnamed field access: `my_point.x`.
    FieldAccess {
        /// The structure or tuple typed expression.
        expr: Box<Self>,
        /// The identifier of the field.
        field: FieldIdentifier,
    },
    /// A lambda expression: `|x, y| x * y`.
    Lambda {
        /// If true, the closure is a `move` closure.
        is_move: bool,
        /// The lambda inputs.
        inputs: Vec<(Ident, Typ)>,
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
        Identifier: ident { identifier: impl Into<Ident> = identifier.into() }
        Identifier: test_ident { identifier: impl AsRef<str> = Loc::test_id(identifier.as_ref()) }
        Some: some {
            expr: Self = Box::new(expr),
        }
        None: none ()
        UnOp: unop {
            op: UOp,
            expr: Self = Box::new(expr),
        }
        BinOp: binop {
            op: BOp,
            lft: Self = lft.into(),
            rgt: Self = rgt.into(),
        }
        IfThenElse: ite {
            cnd: Self = Box::new(cnd),
            thn: Block,
            els: Block,
        }
        MemoryAccess: memory_access { identifier: impl Into<Ident> = identifier.into() }
        InputAccess: input_access { identifier: impl Into<Ident> = identifier.into() }
        Structure: structure {
            name: impl Into<Ident> = name.into(),
            fields: Vec<(Ident, Self)>
        }
        Enumeration: enumeration {
            name: impl Into<Ident> = name.into(),
            element: impl Into<Ident> = element.into(),
        }
        Array: array { elements: Vec<Self> }
        Tuple: tuple { elements: Vec<Self> }
        Block: block { block: Block }
        FunctionCall: function_call {
            function: Self = function.into(),
            arguments: Vec<Self>,
        }
        NodeCall: node_call {
            memory_ident: impl Into<Ident> = memory_ident.into(),
            node_identifier: impl Into<Ident> = node_identifier.into(),
            input_name: impl Into<Ident> = input_name.into(),
            input_fields: Vec<(Ident, Self)>,
        }
        FieldAccess: field_access {
            expr: Self = expr.into(),
            field: FieldIdentifier
        }
        Lambda: lambda {
            is_move: bool,
            inputs: Vec<(Ident, Typ)>,
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
            UnOp { .. }
            | BinOp { .. }
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

    /// True on expressions that require parens to be used as a argument to unary or binary
    /// operations.
    pub fn as_op_arg_requires_parens(&self) -> bool {
        use Expr::*;
        match self {
            BinOp { .. } | IfThenElse { .. } | Lambda { .. } => true,
            Literal { .. }
            | Identifier { .. }
            | UnOp { .. }
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
    /// Transform [ir2] expression into RustAST expression.
    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> syn::Expr {
        match self {
            Self::Literal { literal } => literal.into_syn(),
            Self::Identifier { identifier } => {
                parse_quote! { #identifier }
            }
            Self::Some { expr } => {
                let syn_expr = expr.into_syn(crates);
                parse_quote! { Some(#syn_expr) }
            }
            Self::None => parse_quote! { None },
            Self::MemoryAccess { identifier } => {
                let id = identifier.to_last_var();
                parse_quote!( self.#id )
            }
            Self::InputAccess { identifier } => {
                parse_quote!( input.#identifier )
            }
            Self::Structure { name, fields } => {
                let fields: Vec<syn::FieldValue> = fields
                    .into_iter()
                    .map(|(name, expr)| {
                        let expr = expr.into_syn(crates);
                        parse_quote!(#name : #expr)
                    })
                    .collect();
                let name = format_ident!("{name}");
                parse_quote!(#name { #(#fields),* })
            }
            Self::Enumeration { name, element } => {
                parse_quote!(#name :: #element)
            }
            Self::Array { elements } => {
                let elements = elements.into_iter().map(|expr| expr.into_syn(crates));
                parse_quote! { [#(#elements),*]}
            }
            Self::Tuple { elements } => {
                let elements = elements.into_iter().map(|expr| expr.into_syn(crates));
                parse_quote! { (#(#elements),*) }
            }
            Self::Block { block } => syn::Expr::Block(syn::ExprBlock {
                attrs: vec![],
                label: None,
                block: block.into_syn(crates),
            }),
            Self::FunctionCall {
                function,
                arguments,
            } => {
                let function_parens = function.as_function_requires_parens();
                let function = function.into_syn(crates);
                let arguments = arguments.into_iter().map(|expr| expr.into_syn(crates));
                if function_parens {
                    parse_quote! { (#function)(#(#arguments),*) }
                } else {
                    parse_quote! { #function(#(#arguments),*) }
                }
            }
            Self::UnOp { op, expr } => {
                let op = op.into_syn();
                let expr = expr.into_syn(crates);
                syn::Expr::Unary(parse_quote! { #op (#expr) })
            }
            Self::BinOp { op, lft, rgt } => {
                let left = if lft.as_op_arg_requires_parens() {
                    let expr = lft.into_syn(crates);
                    parse_quote! { (#expr) }
                } else {
                    lft.into_syn(crates)
                };
                let right = if rgt.as_op_arg_requires_parens() {
                    let expr = rgt.into_syn(crates);
                    parse_quote! { (#expr) }
                } else {
                    rgt.into_syn(crates)
                };
                let binary = op.into_syn();
                syn::Expr::Binary(parse_quote! { #left #binary #right })
            }
            Self::NodeCall {
                memory_ident,
                input_name,
                input_fields,
                ..
            } => {
                let ident = memory_ident;
                let receiver: syn::ExprField = parse_quote! { self.#ident};
                let input_fields: Vec<syn::FieldValue> = input_fields
                    .into_iter()
                    .map(|(name, expr)| {
                        let id = name;
                        let expr = expr.into_syn(crates);
                        parse_quote! { #id : #expr }
                    })
                    .collect();

                let input_name = input_name;
                let argument: syn::ExprStruct = parse_quote! { #input_name { #(#input_fields),* }};

                syn::Expr::MethodCall(parse_quote! { #receiver.step (#argument) })
            }
            Self::FieldAccess { expr, field } => {
                let expr = expr.into_syn(crates);
                match field {
                    FieldIdentifier::Named(name) => {
                        parse_quote!(#expr.#name)
                    }
                    FieldIdentifier::Unnamed(number) => {
                        let number: TokenStream2 = format!("{number}").parse().unwrap();
                        parse_quote!(#expr.#number)
                    }
                }
            }
            Self::Lambda {
                is_move,
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
                            ident: identifier,
                            subpat: None,
                        });
                        let pattern = syn::Pat::Type(syn::PatType {
                            attrs: Vec::new(),
                            pat: Box::new(pattern),
                            colon_token: Default::default(),
                            ty: Box::new(typ.into_syn()),
                        });
                        pattern
                    })
                    .collect();
                let capture = if is_move {
                    Some(syn::token::Move {
                        span: Span::call_site(),
                    })
                } else {
                    None
                };
                let closure = syn::ExprClosure {
                    attrs: Vec::new(),
                    asyncness: None,
                    movability: None,
                    capture,
                    or1_token: Default::default(),
                    inputs,
                    or2_token: Default::default(),
                    output: syn::ReturnType::Type(Default::default(), Box::new(output.into_syn())),
                    body: Box::new(body.into_syn(crates)),
                    lifetimes: None,
                    constness: None,
                };
                syn::Expr::Closure(closure)
            }
            Self::IfThenElse { cnd, thn, els } => {
                let cnd = Box::new(cnd.into_syn(crates));
                let thn = thn.into_syn(crates);
                let els = els.into_syn(crates);
                let els = parse_quote! { #els };
                let els = Some((Default::default(), Box::new(els)));
                syn::Expr::If(syn::ExprIf {
                    attrs: Vec::new(),
                    if_token: Default::default(),
                    cond: cnd,
                    then_branch: thn,
                    else_branch: els,
                })
            }
            Self::Match { matched, arms } => {
                let arms = arms
                    .into_iter()
                    .map(|(pattern, guard, body)| syn::Arm {
                        attrs: Vec::new(),
                        pat: pattern.into_syn(),
                        guard: guard
                            .map(|expression| expression.into_syn(crates))
                            .map(|g| (Default::default(), Box::new(g))),
                        body: Box::new(body.into_syn(crates)),
                        fat_arrow_token: Default::default(),
                        comma: Some(Default::default()),
                    })
                    .collect();
                syn::Expr::Match(syn::ExprMatch {
                    attrs: Vec::new(),
                    match_token: Default::default(),
                    expr: Box::new(matched.into_syn(crates)),
                    brace_token: Default::default(),
                    arms,
                })
            }
            Self::Map { mapped, function } => {
                let receiver = Box::new(mapped.into_syn(crates));
                let method = Ident::new("map", Span::call_site());
                let arguments = vec![function.into_syn(crates)];
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
                    receiver: Box::new(folded.into_syn(crates)),
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
                    initialization.into_syn(crates),
                    function.into_syn(crates),
                ]),
                dot_token: Default::default(),
            }),
            Self::Sort { sorted, function } => {
                let token_sorted = sorted.into_syn(crates);
                let token_function = function.into_syn(crates);

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
                    .map(|expression| expression.into_syn(crates));
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

/// Field access member.
#[derive(Debug, PartialEq, Clone)]
pub enum FieldIdentifier {
    /// Named field access.
    Named(Ident),
    /// Unnamed field access.
    Unnamed(usize),
}

impl FieldIdentifier {
    pub fn named(s: impl Into<Ident>) -> Self {
        Self::Named(s.into())
    }
    pub fn unnamed(n: usize) -> Self {
        Self::Unnamed(n)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_literal_from_ir2_literal() {
        let expression = Expr::lit(Constant::Integer(parse_quote!(1i64)));
        let control = parse_quote! { 1i64 };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_identifier_from_ir2_identifier() {
        let expression = Expr::test_ident("x");
        let control = parse_quote! { x };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_self_from_ir2_memory_access() {
        let expression = Expr::memory_access(Loc::test_id("x"));
        let control = parse_quote! { self.last_x};
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_input_from_ir2_input_access() {
        let expression = Expr::input_access(Loc::test_id("i"));
        let control = parse_quote! { input.i};
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_structure_from_ir2_structure() {
        let expression = Expr::structure(
            Loc::test_id("Point"),
            vec![
                (
                    Loc::test_id("x"),
                    Expr::lit(Constant::Integer(parse_quote!(1i64))),
                ),
                (
                    Loc::test_id("y"),
                    Expr::lit(Constant::Integer(parse_quote!(2i64))),
                ),
            ],
        );
        let control = parse_quote! { Point { x : 1i64, y : 2i64 } };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_array_from_ir2_array() {
        let expression = Expr::array(vec![
            Expr::lit(Constant::Integer(parse_quote!(1i64))),
            Expr::lit(Constant::Integer(parse_quote!(2i64))),
        ]);
        let control = parse_quote! { [1i64, 2i64] };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_block_from_ir2_block() {
        let expression = Expr::block(Block {
            statements: vec![
                Stmt::Let {
                    pattern: Pattern::test_ident("x"),
                    expr: Expr::lit(Constant::Integer(parse_quote!(1i64))),
                },
                Stmt::ExprLast {
                    expr: Expr::test_ident("x"),
                },
            ],
        });
        let control = parse_quote! { { let x = 1i64; x } };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_function_call_from_ir2_function_call() {
        let expression = Expr::function_call(
            Expr::test_ident("foo"),
            vec![Expr::test_ident("a"), Expr::test_ident("b")],
        );

        let control = parse_quote! { foo (a, b) };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_binary_from_ir2_function_call() {
        let expression = Expr::binop(BOp::Add, Expr::test_ident("a"), Expr::test_ident("b"));

        let control = parse_quote! { a + b };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_method_call_from_ir2_node_call() {
        let expression = Expr::node_call(
            Loc::test_id("node_state"),
            Loc::test_id("node"),
            Loc::test_id("NodeInput"),
            vec![(
                Loc::test_id("i"),
                Expr::Literal {
                    literal: Constant::Integer(parse_quote!(1i64)),
                },
            )],
        );

        let control = parse_quote! { self.node_state.step ( NodeInput { i : 1i64 }) };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_field_access_from_ir2_field_access() {
        let expression = Expr::field_access(
            Expr::test_ident("my_point"),
            FieldIdentifier::Named(Loc::test_id("x")),
        );

        let control = parse_quote! { my_point.x };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_closure_from_ir2_lambda() {
        let expression = Expr::lambda(
            true,
            vec![(Loc::test_id("x"), Typ::int())],
            Typ::int(),
            Expr::block(Block {
                statements: vec![
                    Stmt::let_binding(Pattern::test_ident("y"), Expr::test_ident("x")),
                    Stmt::expression_last(Expr::test_ident("y")),
                ],
            }),
        );

        let control = parse_quote! { move |x: i64| -> i64 { let y = x; y } };
        assert_eq!(expression.into_syn(&mut Default::default()), control,)
    }

    #[test]
    fn should_create_rust_ast_if_then_else_from_ir2_if_then_else() {
        let expression = Expr::ite(
            Expr::test_ident("test"),
            Block::new(vec![Stmt::expression_last(Expr::lit(Constant::int(
                parse_quote!(1i64),
            )))]),
            Block::new(vec![Stmt::expression_last(Expr::lit(Constant::int(
                parse_quote!(0i64),
            )))]),
        );

        let control = parse_quote! { if test { 1i64 } else { 0i64 } };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_match_from_ir2_match() {
        let expression = Expr::pat_match(
            Expr::test_ident("my_color"),
            vec![
                (
                    Pattern::enumeration(Loc::test_id("Color"), Loc::test_id("Blue"), None),
                    None,
                    Expr::lit(Constant::Integer(parse_quote!(1i64))),
                ),
                (
                    Pattern::enumeration(Loc::test_id("Color"), Loc::test_id("Green"), None),
                    None,
                    Expr::Literal {
                        literal: Constant::Integer(parse_quote!(0i64)),
                    },
                ),
            ],
        );

        let control =
            parse_quote! { match my_color { Color::Blue => 1i64, Color::Green => 0i64, } };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_map_operation_from_ir2_map() {
        let expression = Expr::map(Expr::test_ident("a"), Expr::test_ident("f"));

        let control = parse_quote! { a.map (f) };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_fold_iterator_from_ir2_fold() {
        let expression = Expr::fold(
            Expr::test_ident("a"),
            Expr::lit(Constant::Integer(parse_quote!(0i64))),
            Expr::test_ident("sum"),
        );

        let control = parse_quote! { a.into_iter().fold(0i64, sum) };
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_sort_iterator_from_ir2_sort() {
        let expression = Expr::sort(Expr::test_ident("a"), Expr::test_ident("compare"));

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
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_macro_from_ir2_zip() {
        let expression = Expr::zip(vec![Expr::test_ident("a"), Expr::test_ident("b")]);

        let control = parse_quote!({
            let mut iter = itertools::izip!(a, b);
            std::array::from_fn(|_| iter.next().unwrap())
        });
        assert_eq!(expression.into_syn(&mut Default::default()), control)
    }
}
