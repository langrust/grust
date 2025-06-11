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
    /// An external path.
    Path {
        /// The path.
        path: syn::Path,
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
        /// Path to call component from.
        path_opt: Option<syn::Path>,
    },
    /// A named or unnamed field access: `my_point.x`.
    FieldAccess {
        /// The structure or tuple typed expression.
        expr: Box<Self>,
        /// The identifier of the field.
        field: FieldIdentifier,
    },
    /// A array access: `my_array[idx]`.
    ArrayAccess {
        /// The array expression.
        expr: Box<Self>,
        /// The index to access.
        index: syn::LitInt,
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
    MatchExpr {
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
            path_opt: Option<syn::Path>,
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
        MatchExpr: match_expr {
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
            | Path { .. }
            | Some { .. }
            | None { .. }
            | MemoryAccess { .. }
            | InputAccess { .. }
            | Enumeration { .. }
            | Array { .. }
            | Tuple { .. }
            | Block { .. }
            | FieldAccess { .. }
            | ArrayAccess { .. } => false,
            UnOp { .. }
            | BinOp { .. }
            | IfThenElse { .. }
            | Structure { .. }
            | FunctionCall { .. }
            | NodeCall { .. }
            | Lambda { .. }
            | MatchExpr { .. }
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
            | Path { .. }
            | UnOp { .. }
            | Some { .. }
            | None { .. }
            | FunctionCall { .. }
            | NodeCall { .. }
            | MemoryAccess { .. }
            | InputAccess { .. }
            | FieldAccess { .. }
            | ArrayAccess { .. }
            | Enumeration { .. }
            | Structure { .. }
            | Array { .. }
            | Tuple { .. }
            | Block { .. }
            | MatchExpr { .. }
            | Map { .. }
            | Fold { .. }
            | Sort { .. }
            | Zip { .. } => false,
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Literal { literal } => literal.to_tokens(tokens),
            Self::Identifier { identifier } => identifier.to_tokens(tokens),
            Self::Path { path } => path.to_tokens(tokens),
            Self::Some { expr } => quote!(Some(#expr)).to_tokens(tokens),
            Self::None => quote!(None).to_tokens(tokens),
            Self::MemoryAccess { identifier } => {
                let id = identifier.to_last_var();
                quote!(self.#id).to_tokens(tokens)
            }
            Self::InputAccess { identifier } => quote!(input.#identifier).to_tokens(tokens),
            Self::Structure { name, fields } => {
                let fields = fields.iter().map(|(name, expr)| quote!(#name: #expr));
                quote!( #name { #(#fields),* } ).to_tokens(tokens)
            }
            Self::Enumeration { name, element } => tokens.extend(quote!(#name :: #element)),
            Self::Array { elements } => quote!( [#(#elements),*] ).to_tokens(tokens),
            Self::Tuple { elements } => quote!( (#(#elements),*) ).to_tokens(tokens),
            Self::Block { block } => block.to_tokens(tokens),
            Self::FunctionCall {
                function,
                arguments,
            } => {
                let function_parens = function.as_function_requires_parens();
                if function_parens {
                    quote!( (#function)(#(#arguments),*) ).to_tokens(tokens)
                } else {
                    quote!( #function(#(#arguments),*) ).to_tokens(tokens)
                }
            }
            Self::UnOp { op, expr } => tokens.extend(quote!(#op (#expr))),
            Self::BinOp { op, lft, rgt } => {
                if lft.as_op_arg_requires_parens() {
                    quote!((#lft)).to_tokens(tokens)
                } else {
                    lft.to_tokens(tokens)
                }
                op.to_tokens(tokens);
                if rgt.as_op_arg_requires_parens() {
                    quote!((#rgt)).to_tokens(tokens)
                } else {
                    rgt.to_tokens(tokens)
                }
            }
            Self::NodeCall {
                memory_ident,
                input_fields,
                path_opt,
                node_identifier: name,
                ..
            } => {
                let state_ty = name.to_state_ty();
                let input_ty = name.to_input_ty();
                let input_fields = input_fields.iter().map(|(name, expr)| {
                    quote! { #name : #expr }
                });
                if let Some(mut path) = path_opt.clone() {
                    path.segments.pop();
                    let mut state_path = path.clone();
                    let mut input_path = path;
                    state_path.segments.push(state_ty.into());
                    input_path.segments.push(input_ty.into());
                    quote! {
                        <#state_path as grust::core::Component>::step(
                            &mut self.#memory_ident, #input_path { #(#input_fields),* }
                        )
                    }
                    .to_tokens(tokens)
                } else {
                    quote! {
                        <#state_ty as grust::core::Component>::step(
                            &mut self.#memory_ident, #input_ty { #(#input_fields),* }
                        )
                    }
                    .to_tokens(tokens)
                }
            }
            Self::FieldAccess { expr, field } => tokens.extend(quote!(#expr . #field)),
            Self::ArrayAccess { expr, index } => tokens.extend(quote!(#expr[#index])),
            Self::Lambda {
                is_move,
                inputs,
                output,
                body,
            } => {
                let inputs = inputs.iter().map(|(ident, typ)| quote!(#ident : #typ));
                let capture = if *is_move { Some(quote!(move)) } else { None };
                if let Expr::Block { .. } = &**body {
                    quote!(#capture | #(#inputs),* | -> #output #body)
                } else {
                    quote!(#capture | #(#inputs),* | -> #output { #body })
                }
                .to_tokens(tokens)
            }
            Self::IfThenElse { cnd, thn, els } => quote!(if #cnd #thn else #els).to_tokens(tokens),
            Self::MatchExpr { matched, arms } => {
                let arms = arms.iter().map(|(pat, guard_opt, code)| {
                    if let Some(guard) = guard_opt.as_ref() {
                        quote!(#pat if #guard => #code)
                    } else {
                        quote!(#pat => #code)
                    }
                });
                quote! {
                    match #matched {
                        #(#arms,)*
                    }
                }
                .to_tokens(tokens)
            }
            Self::Map { mapped, function } => quote! {
                #mapped . map ( #function )
            }
            .to_tokens(tokens),
            Self::Fold {
                folded,
                initialization,
                function,
            } => quote! {
                #folded . into_iter() . fold( #initialization, #function )
            }
            .to_tokens(tokens),
            Self::Sort { sorted, function } => quote! {
                {
                    let mut grust_reserved_sort = #sorted . clone();
                    grust_reserved_sort.sort_by(|a, b| {
                        let cmp = #function(*a, *b);
                        if cmp < 0 {
                            std::cmp::Ordering::Less
                        } else if 0 < cmp {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });
                    grust_reserved_sort
                }
            }
            .to_tokens(tokens),
            Self::Zip { arrays } => quote! {
                std::array::from_fn(|n| ( #(#arrays[n]),* ))
            }
            .to_tokens(tokens),
        }
    }
}

impl ToLogicTokens for Expr {
    fn to_logic_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Literal { literal } => literal.to_logic_tokens(tokens),
            Self::Identifier { identifier } => identifier.to_tokens(tokens),
            Self::Path { path } => path.to_tokens(tokens),
            Self::Some { expr } => {
                let expr = expr.to_logic();
                quote!(Some(#expr)).to_tokens(tokens)
            }
            Self::None => quote!(None).to_tokens(tokens),
            Self::MemoryAccess { identifier } => {
                let id = identifier.to_last_var();
                quote!(self.#id).to_tokens(tokens)
            }
            Self::InputAccess { identifier } => quote!(input.#identifier).to_tokens(tokens),
            Self::Structure { name, fields } => {
                let fields = fields.iter().map(|(name, expr)| quote!(#name : #expr));
                quote!(#name { #(#fields),* }).to_tokens(tokens)
            }
            Self::Enumeration { name, element } => quote!(#name :: #element).to_tokens(tokens),
            Self::Array { elements } => {
                let elms = elements.iter().map(|e| e.to_logic());
                quote!( [#(#elms),*] ).to_tokens(tokens)
            }
            Self::Tuple { elements } => {
                let elms = elements.iter().map(|e| e.to_logic());
                quote!( (#(#elms),*) ).to_tokens(tokens)
            }
            Self::Block { block } => block.to_logic_tokens(tokens),
            Self::FunctionCall {
                function,
                arguments,
            } => {
                let arguments = arguments.iter().map(|arg| arg.to_logic());
                if function.as_function_requires_parens() {
                    quote!( (#function)(#(#arguments),*) ).to_tokens(tokens)
                } else {
                    quote!( #function(#(#arguments),*) ).to_tokens(tokens)
                }
            }
            Self::UnOp { op, expr } => quote!(#op #expr).to_tokens(tokens),
            Self::BinOp { op, lft, rgt } => {
                if lft.as_op_arg_requires_parens() {
                    let expr = lft.to_logic();
                    quote!( (#expr) ).to_tokens(tokens)
                } else {
                    lft.to_logic_tokens(tokens)
                }
                op.to_tokens(tokens);
                if rgt.as_op_arg_requires_parens() {
                    let expr = rgt.to_logic();
                    quote!( (#expr) ).to_tokens(tokens)
                } else {
                    rgt.to_logic_tokens(tokens)
                }
            }
            Self::NodeCall {
                memory_ident,
                node_identifier: name,
                input_fields,
                path_opt,
                ..
            } => {
                let state_ty = name.to_state_ty();
                let input_ty = name.to_input_ty();
                let input_fields = input_fields.iter().map(|(field, expr)| {
                    let expr = expr.to_logic();
                    quote!(#field : #expr)
                });
                if let Some(mut path) = path_opt.clone() {
                    path.segments.pop();
                    quote! {
                        <#path::#state_ty as grust::core::Component>::step(
                            &mut self.#memory_ident, #path::#input_ty { #(#input_fields),* }
                        )
                    }
                    .to_tokens(tokens)
                } else {
                    quote! {
                        <#state_ty as grust::core::Component>::step(
                            &mut self.#memory_ident, #input_ty { #(#input_fields),* }
                        )
                    }
                    .to_tokens(tokens)
                }
            }
            Self::FieldAccess { expr, field } => quote!(#expr . #field).to_tokens(tokens),
            Self::ArrayAccess { expr, index } => quote!(#expr[#index]).to_tokens(tokens),
            Self::Lambda {
                is_move,
                inputs,
                output,
                body,
            } => {
                let inputs = inputs.iter().map(|(ident, typ)| {
                    let typ = typ.to_logic();
                    quote!(#ident : #typ)
                });
                let output = output.to_logic();
                let body = body.to_logic();
                let capture = if *is_move { Some(quote!(move)) } else { None };
                quote!(#capture | #(#inputs),* | -> #output { #body }).to_tokens(tokens)
            }
            Self::IfThenElse { cnd, thn, els } => {
                let (cnd, thn, els) = (cnd.to_logic(), thn.to_logic(), els.to_logic());
                quote!(if #cnd #thn else #els).to_tokens(tokens)
            }
            Self::MatchExpr { matched, arms } => {
                let matched = matched.to_logic();
                let arms = arms.iter().map(|(pat, guard_opt, code)| {
                    let code = code.to_logic();
                    let guard_opt = guard_opt.as_ref().map(|g| {
                        let g = g.to_logic();
                        quote!(if #g)
                    });
                    quote!(#pat #guard_opt => #code)
                });
                quote! {
                    match #matched { #(#arms),* }
                }
                .to_tokens(tokens)
            }
            Self::Map { mapped, function } => {
                let mapped = mapped.to_logic();
                let function = function.to_logic();
                quote! {
                    #mapped . map ( #function )
                }
                .to_tokens(tokens)
            }
            Self::Fold {
                folded,
                initialization,
                function,
            } => {
                let folded = folded.to_logic();
                let init = initialization.to_logic();
                let function = function.to_logic();
                quote!(#folded . fold ( #init, #function )).to_tokens(tokens)
            }
            Self::Sort { sorted, function } => quote! {
                let sorted = sorted.to_logic();
                let function = function.to_logic();
                {
                    let mut grust_reserved_sort = #sorted . clone();
                    grust_reserved_sort.sort_by(|a, b| {
                        let cmp = #function(*a, *b);
                        if cmp < 0 {
                            std::cmp::Ordering::Less
                        } else if 0 < cmp {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });
                    grust_reserved_sort
                }
            }
            .to_tokens(tokens),
            Self::Zip { arrays } => {
                let arrays = arrays.iter().map(|a| a.to_logic());
                quote! {
                    std::array::from_fn(|n| ( #(#arrays[n]),* ))
                }
                .to_tokens(tokens)
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

impl ToTokens for FieldIdentifier {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Named(ident) => ident.to_tokens(tokens),
            Self::Unnamed(n) => n.to_tokens(tokens),
        }
    }
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
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_identifier_from_ir2_identifier() {
        let expression = Expr::test_ident("x");
        let control = parse_quote! { x };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_self_from_ir2_memory_access() {
        let expression = Expr::memory_access(Loc::test_id("x"));
        let control = parse_quote! { self.last_x};
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_field_access_to_input_from_ir2_input_access() {
        let expression = Expr::input_access(Loc::test_id("i"));
        let control = parse_quote! { input.i};
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
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
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_array_from_ir2_array() {
        let expression = Expr::array(vec![
            Expr::lit(Constant::Integer(parse_quote!(1i64))),
            Expr::lit(Constant::Integer(parse_quote!(2i64))),
        ]);
        let control = parse_quote! { [1i64, 2i64] };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
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
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_function_call_from_ir2_function_call() {
        let expression = Expr::function_call(
            Expr::test_ident("foo"),
            vec![Expr::test_ident("a"), Expr::test_ident("b")],
        );

        let control = parse_quote! { foo (a, b) };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_binary_from_ir2_function_call() {
        let expression = Expr::binop(BOp::Add, Expr::test_ident("a"), Expr::test_ident("b"));

        let control = parse_quote! { a + b };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
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
            None,
        );

        let control = parse_quote! { <NodeState as grust::core::Component>::step(&mut self.node_state, NodeInput { i : 1i64 }) };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_method_call_from_ir2_node_call_with_path() {
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
            Some(parse_quote!(path::to::node)),
        );

        let control = parse_quote! { <path::to::NodeState as grust::core::Component>::step(&mut self.node_state, path::to::NodeInput { i : 1i64 }) };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_field_access_from_ir2_field_access() {
        let expression = Expr::field_access(
            Expr::test_ident("my_point"),
            FieldIdentifier::Named(Loc::test_id("x")),
        );

        let control = parse_quote! { my_point.x };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
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
                    Stmt::expr_last(Expr::test_ident("y")),
                ],
            }),
        );

        let control: syn::Expr = parse_quote! { move |x: i64| -> i64 { let y = x; y } };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_if_then_else_from_ir2_if_then_else() {
        let expression = Expr::ite(
            Expr::test_ident("test"),
            Block::new(vec![Stmt::expr_last(Expr::lit(Constant::int(
                parse_quote!(1i64),
            )))]),
            Block::new(vec![Stmt::expr_last(Expr::lit(Constant::int(
                parse_quote!(0i64),
            )))]),
        );

        let control = parse_quote! { if test { 1i64 } else { 0i64 } };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_match_from_ir2_match() {
        let expression = Expr::match_expr(
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

        let control: syn::Expr =
            parse_quote! { match my_color { Color::Blue => 1i64, Color::Green => 0i64, } };
        println!("{}", expression.to_token_stream());
        println!("----");
        println!("{}", control.to_token_stream());
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_map_operation_from_ir2_map() {
        let expression = Expr::map(Expr::test_ident("a"), Expr::test_ident("f"));

        let control = parse_quote! { a.map (f) };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_fold_iterator_from_ir2_fold() {
        let expression = Expr::fold(
            Expr::test_ident("a"),
            Expr::lit(Constant::Integer(parse_quote!(0i64))),
            Expr::test_ident("sum"),
        );

        let control: syn::Expr = parse_quote! { a.into_iter().fold(0i64, sum) };
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control)
    }

    #[test]
    fn should_create_rust_ast_sort_iterator_from_ir2_sort() {
        let expression = Expr::sort(Expr::test_ident("a"), Expr::test_ident("compare"));

        let control: syn::Expr = parse_quote!({
            let mut grust_reserved_sort = a.clone();
            grust_reserved_sort.sort_by(|a, b| {
                let cmp = compare(*a, *b);
                if cmp < 0 {
                    std::cmp::Ordering::Less
                } else if 0 < cmp {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            grust_reserved_sort
        });
        println!("{}", expression.to_token_stream());
        println!("----");
        println!("{}", control.to_token_stream());
        let expr: syn::Expr = parse_quote!(#expression);
        assert_eq!(expr, control);
    }

    #[test]
    fn should_create_rust_ast_macro_from_ir2_zip() {
        let expression = Expr::zip(vec![Expr::test_ident("a"), Expr::test_ident("b")]);

        let control: syn::Expr = parse_quote!(std::array::from_fn(|n| (a[n], b[n])));
        let expr: syn::Expr = parse_quote!(#expression);
        println!("{}", expr.to_token_stream());
        println!("----");
        println!("{}", control.to_token_stream());
        assert_eq!(expr, control);

        let expression = Expr::zip(vec![
            Expr::test_ident("a"),
            Expr::test_ident("b"),
            Expr::test_ident("c"),
            Expr::test_ident("d"),
        ]);

        let control: syn::Expr = parse_quote!(std::array::from_fn(|n| (a[n], b[n], c[n], d[n])));
        let expr: syn::Expr = parse_quote!(#expression);
        println!("\n\n{}", expr.to_token_stream());
        println!("----");
        println!("{}", control.to_token_stream());
        assert_eq!(expr, control)
    }
}
