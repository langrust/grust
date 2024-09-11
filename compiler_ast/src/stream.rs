prelude! {
    syn::parse::Parse,
    equation::EventPattern,
    expr::*,
    operator::BinaryOperator,
}

/// Initialized buffer stream expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Fby {
    /// The initialization constant.
    pub constant: Box<Expr>,
    /// The buffered expression.
    pub expression: Box<Expr>,
}
mk_new! { impl Fby =>
    new {
        constant: impl Into<Box<Expr >> = constant.into(),
        expression: impl Into<Box<Expr >> = expression.into(),
    }
}
impl Fby {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::fby)
    }
    pub fn parse(constant: Expr, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: keyword::fby = input.parse()?;
        let expression: Expr = input.parse()?;
        Ok(Fby::new(constant, expression))
    }
}

/// Pattern matching for event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct When {
    /// The pattern receiving the value of the event.
    pub pattern: EventPattern,
    pub then_token: keyword::then,
    /// Action triggered by event.
    pub expression: Box<Expr>,
}
mk_new! { impl When =>
    new {
        pattern: EventPattern,
        then_token: keyword::then,
        expression: impl Into<Box<Expr >> = expression.into(),
    }
}
impl When {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::when)
    }
}
impl Parse for When {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: keyword::when = input.parse()?;
        let pattern: EventPattern = input.parse()?;
        let then_token: keyword::then = input.parse()?;
        let expression: Expr = input.parse()?;
        Ok(When::new(pattern, then_token, expression))
    }
}

/// Emit event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Emit {
    pub emit_token: keyword::emit,
    /// The expression to emit.
    pub expr: Box<Expr>,
}
mk_new! { impl Emit =>
    new {
        emit_token: keyword::emit,
        expr: impl Into<Box<Expr >> = expr.into(),
    }
}
impl Emit {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::emit)
    }
}
impl Parse for Emit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let emit_token: keyword::emit = input.parse()?;
        let expr: Expr = input.parse()?;
        Ok(Emit::new(emit_token, expr))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust stream expression kind AST.
pub enum Expr {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(String),
    /// Application expression.
    Application(Application<Self>),
    /// Unop expression.
    Unop(Unop<Self>),
    /// Binop expression.
    Binop(Binop<Self>),
    /// IfThenElse expression.
    IfThenElse(IfThenElse<Self>),
    /// Abstraction expression with inputs types.
    TypedAbstraction(TypedAbstraction<Self>),
    /// Structure expression.
    Structure(Structure<Self>),
    /// Tuple expression.
    Tuple(Tuple<Self>),
    /// Enumeration expression.
    Enumeration(Enumeration<Self>),
    /// Array expression.
    Array(Array<Self>),
    /// Pattern matching expression.
    Match(Match<Self>),
    /// Field access expression.
    FieldAccess(FieldAccess<Self>),
    /// Tuple element access expression.
    TupleElementAccess(TupleElementAccess<Self>),
    /// Array map operator expression.
    Map(Map<Self>),
    /// Array fold operator expression.
    Fold(Fold<Self>),
    /// Array sort operator expression.
    Sort(Sort<Self>),
    /// Arrays zip operator expression.
    Zip(Zip<Self>),
    /// Initialized buffer stream expression.
    Fby(Fby),
    /// Pattern matching event.
    When(When),
    /// Emit event.
    Emit(Emit),
}

mk_new! { impl Expr =>
    Constant: cst(arg: Constant = arg)
    Identifier: ident(arg : impl Into<String> = arg.into())
    Application: app(arg : Application<Self> = arg)
    Unop: unop(arg: Unop<Self> = arg)
    Binop: binop(arg: Binop<Self> = arg)
    IfThenElse: ite(arg: IfThenElse<Self> = arg)
    TypedAbstraction: type_abstraction(arg: TypedAbstraction<Self> = arg)
    Structure: structure(arg: Structure<Self> = arg)
    Tuple: tuple(arg: Tuple<Self> = arg)
    Enumeration: enumeration(arg: Enumeration<Self> = arg)
    Array: array(arg: Array<Self> = arg)
    Match: pat_match(arg: Match<Self> = arg)
    FieldAccess: field_access(arg: FieldAccess<Self> = arg)
    TupleElementAccess: tuple_access(arg: TupleElementAccess<Self> = arg)
    Map: map(arg: Map<Self> = arg)
    Fold: fold(arg: Fold<Self> = arg)
    Sort: sort(arg: Sort<Self> = arg)
    Zip: zip(arg: Zip<Self> = arg)
    Fby: fby(arg: Fby = arg)
    When: when_match(arg: When = arg)
    Emit: emit(arg: Emit = arg)
}

impl ParsePrec for Expr {
    fn parse_term(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = if input.fork().call(Constant::parse).is_ok() {
            Self::Constant(input.parse()?)
        } else if Unop::<Self>::peek(input) {
            Self::Unop(input.parse()?)
        } else if Zip::<Self>::peek(input) {
            Self::Zip(input.parse()?)
        } else if Match::<Self>::peek(input) {
            Self::Match(input.parse()?)
        } else if Tuple::<Self>::peek(input) {
            let mut tuple: Tuple<Self> = input.parse()?;
            if tuple.elements.len() == 1 {
                tuple.elements.pop().unwrap()
            } else {
                Self::Tuple(tuple)
            }
        } else if Array::<Self>::peek(input) {
            Self::Array(input.parse()?)
        } else if Structure::<Self>::peek(input) {
            Self::Structure(input.parse()?)
        } else if Enumeration::<Self>::peek(input) {
            Self::Enumeration(input.parse()?)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Self::Identifier(ident.to_string())
        } else {
            return Err(input.error("expected expression"));
        };
        loop {
            if Sort::<Self>::peek(input) {
                expression = Self::Sort(Sort::<Self>::parse(expression, input)?);
            } else if Map::<Self>::peek(input) {
                expression = Self::Map(Map::<Self>::parse(expression, input)?)
            } else if Fold::<Self>::peek(input) {
                expression = Self::Fold(Fold::<Self>::parse(expression, input)?)
            } else if TupleElementAccess::<Self>::peek(input) {
                expression =
                    Self::TupleElementAccess(TupleElementAccess::<Self>::parse(expression, input)?)
            } else if FieldAccess::<Self>::peek(input) {
                expression = Self::FieldAccess(FieldAccess::<Self>::parse(expression, input)?)
            } else if Application::<Self>::peek(input) {
                expression = Self::Application(Application::<Self>::parse(expression, input)?)
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn parse_prec1(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Self::parse_term(input)?;

        loop {
            if BinaryOperator::peek_prec1(input) {
                expression = Self::Binop(Binop::<Self>::parse_term(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec2(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Self::parse_prec1(input)?;

        loop {
            if BinaryOperator::peek_prec2(input) {
                expression = Self::Binop(Binop::<Self>::parse_prec1(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec3(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Self::parse_prec2(input)?;

        loop {
            if BinaryOperator::peek_prec3(input) {
                expression = Self::Binop(Binop::<Self>::parse_prec2(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec4(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Self::parse_prec3(input)?;

        loop {
            if BinaryOperator::peek_prec4(input) {
                expression = Self::Binop(Binop::<Self>::parse_prec3(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
}
impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = if TypedAbstraction::<Self>::peek(input) {
            Self::TypedAbstraction(input.parse()?)
        } else if IfThenElse::<Self>::peek(input) {
            Self::IfThenElse(input.parse()?)
        } else if When::peek(input) {
            Self::When(input.parse()?)
        } else if Emit::peek(input) {
            Self::Emit(input.parse()?)
        } else {
            Self::parse_prec4(input)?
        };
        loop {
            if Fby::peek(input) {
                expression = Self::Fby(Fby::parse(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
}

#[cfg(test)]
mod parse_stream_expression {
    prelude! {
        expr::{
            Application, Arm, Array, Binop, Enumeration, FieldAccess, Fold, Map, Match, Sort,
            Structure, Tuple, TupleElementAccess, TypedAbstraction, Zip, PatStructure, Pattern
        },
        equation::{EventPattern, LetEventPattern},
        stream::{Fby, Expr, Emit, When},
        operator::BinaryOperator,
        quote::format_ident,
    }

    #[test]
    fn should_parse_followed_by() {
        let expression: Expr = syn::parse_quote! {0 fby p.x};
        let control = Expr::fby(Fby::new(
            Expr::cst(Constant::int(syn::parse_quote! {0})),
            Expr::field_access(FieldAccess::new(Expr::ident("p"), "x")),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_constant() {
        let expression: Expr = syn::parse_quote! {1};
        let control = Expr::cst(Constant::int(syn::parse_quote! {1}));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_identifier() {
        let expression: Expr = syn::parse_quote! {x};
        let control = Expr::ident("x");
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_application() {
        let expression: Expr = syn::parse_quote! {f(x)};
        let control = Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop() {
        let expression: Expr = syn::parse_quote! {a+b};
        let control = Expr::binop(Binop::new(
            BinaryOperator::Add,
            Expr::ident("a"),
            Expr::ident("b"),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop_with_precedence() {
        let expression: Expr = syn::parse_quote! {a+b*c};
        let control = Expr::binop(Binop::new(
            BinaryOperator::Add,
            Expr::ident("a"),
            Expr::Binop(Binop::new(
                BinaryOperator::Mul,
                Expr::ident("b"),
                Expr::ident("c"),
            )),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_typed_abstraction() {
        let expression: Expr = syn::parse_quote! {|x: int| f(x)};
        let control = Expr::type_abstraction(TypedAbstraction::new(
            vec![("x".into(), Typ::int())],
            Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")])),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_structure() {
        let expression: Expr = syn::parse_quote! {Point {x: 0, y: 1}};
        let control = Expr::structure(Structure::new(
            "Point",
            vec![
                ("x".into(), Expr::cst(Constant::int(syn::parse_quote! {0}))),
                ("y".into(), Expr::cst(Constant::int(syn::parse_quote! {1}))),
            ],
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple() {
        let expression: Expr = syn::parse_quote! {(x, 0)};
        let control = Expr::tuple(Tuple::new(vec![
            Expr::ident("x"),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
        ]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_enumeration() {
        let expression: Expr = syn::parse_quote! {Color::Pink};
        let control = Expr::enumeration(Enumeration::new("Color", "Pink"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_array() {
        let expression: Expr = syn::parse_quote! {[1, 2, 3]};
        let control = Expr::array(Array::new(vec![
            Expr::cst(Constant::int(syn::parse_quote! {1})),
            Expr::cst(Constant::int(syn::parse_quote! {2})),
            Expr::cst(Constant::int(syn::parse_quote! {3})),
        ]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_match() {
        let expression: Expr = syn::parse_quote! {
            match a {
                Point {x: 0, y: _} => 0,
                Point {x: x, y: _} if f(x) => -1,
                _ => 1,
            }
        };
        let control = Expr::pat_match(Match::new(
            Expr::ident("a"),
            vec![
                Arm::new(
                    Pattern::Structure(PatStructure::new(
                        "Point",
                        vec![
                            (
                                "x".into(),
                                Some(Pattern::Constant(Constant::int(syn::parse_quote! {0}))),
                            ),
                            ("y".into(), Some(Pattern::Default)),
                        ],
                        None,
                    )),
                    Expr::cst(Constant::int(syn::parse_quote! {0})),
                ),
                Arm {
                    pattern: Pattern::Structure(PatStructure::new(
                        "Point",
                        vec![
                            ("x".into(), Some(Pattern::ident("x"))),
                            ("y".into(), Some(Pattern::Default)),
                        ],
                        None,
                    )),
                    guard: Some(Expr::app(Application::new(
                        Expr::ident("f"),
                        vec![Expr::ident("x")],
                    ))),
                    expression: Expr::cst(Constant::int(syn::parse_quote! {-1})),
                },
                Arm::new(
                    Pattern::Default,
                    Expr::cst(Constant::int(syn::parse_quote! {1})),
                ),
            ],
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_field_access() {
        let expression: Expr = syn::parse_quote! {p.x};
        let control = Expr::field_access(FieldAccess::new(Expr::ident("p"), "x"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple_element_access() {
        let expression: Expr = syn::parse_quote! {t.0};
        let control = Expr::tuple_access(TupleElementAccess::new(Expr::ident("t"), 0));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_map() {
        let expression: Expr = syn::parse_quote! {a.map(f)};
        let control = Expr::map(Map::new(Expr::ident("a"), Expr::ident("f")));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_fold() {
        let expression: Expr = syn::parse_quote! {a.fold(0, sum)};
        let control = Expr::fold(Fold::new(
            Expr::ident("a"),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
            Expr::ident("sum"),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_sort() {
        let expression: Expr = syn::parse_quote! {a.sort(order)};
        let control = Expr::sort(Sort::new(Expr::ident("a"), Expr::ident("order")));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_zip() {
        let expression: Expr = syn::parse_quote! {zip(a, b, c)};
        let control = Expr::zip(Zip::new(vec![
            Expr::ident("a"),
            Expr::ident("b"),
            Expr::ident("c"),
        ]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_emit() {
        let expression: Expr = syn::parse_quote! {emit 0};
        let control = Expr::emit(Emit::new(
            Default::default(),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_when() {
        let expression: Expr = syn::parse_quote! {when let d = p? then emit x};
        let control = Expr::when_match(When::new(
            EventPattern::Let(LetEventPattern::new(
                Default::default(),
                expr::Pattern::ident("d"),
                Default::default(),
                format_ident!("p"),
                Default::default(),
            )),
            Default::default(),
            Expr::emit(Emit::new(Default::default(), Expr::ident("x"))),
        ));
        assert_eq!(expression, control)
    }
}
