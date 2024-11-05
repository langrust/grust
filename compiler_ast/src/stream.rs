prelude! {
    syn::{Parse, Token},
    equation::EventPattern,
    expr::*,
    operator::BinaryOperator,
}

/// Buffered signal.
#[derive(Debug, PartialEq, Clone)]
pub struct Last {
    /// Signal identifier.
    pub ident: Ident,
    /// The initialization constant.
    pub constant: Option<Box<Expr>>,
}
mk_new! { impl Last =>
    new {
        ident: Ident,
        constant: Option<Expr> = constant.map(Expr::into),
    }
}
impl Last {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::last)
    }
}
impl Parse for Last {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let _: keyword::last = input.parse()?;
        let ident = input.parse()?;
        let constant = if input.peek(keyword::init) {
            let _: keyword::init = input.parse()?;
            let constant = input.parse()?;
            Some(constant)
        } else {
            None
        };
        Ok(Last::new(ident, constant))
    }
}

/// Pattern matching for event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct When {
    /// The pattern receiving the value of the event.
    pub pattern: EventPattern,
    /// The optional guard.
    pub guard: Option<Box<Expr>>,
    pub then_token: keyword::then,
    /// Action triggered by event.
    pub expression: Box<Expr>,
}
mk_new! { impl When =>
    new {
        pattern: EventPattern,
        guard: Option<Expr> = guard.map(Expr::into),
        then_token: keyword::then,
        expression: impl Into<Box<Expr >> = expression.into(),
    }
}
impl When {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::when)
    }
}
impl Parse for When {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let _: keyword::when = input.parse()?;
        let pattern: EventPattern = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let _: Token![if] = input.parse()?;
                let guard = input.parse()?;
                Some(guard)
            } else {
                None
            }
        };
        let then_token: keyword::then = input.parse()?;
        let expression: Expr = input.parse()?;
        Ok(When::new(pattern, guard, then_token, expression))
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
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::emit)
    }
}
impl Parse for Emit {
    fn parse(input: ParseStream) -> syn::Res<Self> {
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
    Last(Last),
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
    Last: last(arg: Last = arg)
    Emit: emit(arg: Emit = arg)
}
impl ParsePrec for Expr {
    fn parse_term(input: ParseStream) -> syn::Res<Self> {
        let mut expression = if input.fork().call(Constant::parse).is_ok() {
            Self::Constant(input.parse()?)
        } else if Last::peek(input) {
            Self::Last(input.parse()?)
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
        } else if input.fork().call(Ident::parse).is_ok() {
            let ident: Ident = input.parse()?;
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

    fn parse_prec1(input: ParseStream) -> syn::Res<Self> {
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
    fn parse_prec2(input: ParseStream) -> syn::Res<Self> {
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
    fn parse_prec3(input: ParseStream) -> syn::Res<Self> {
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
    fn parse_prec4(input: ParseStream) -> syn::Res<Self> {
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
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let expression = if TypedAbstraction::<Self>::peek(input) {
            Self::TypedAbstraction(input.parse()?)
        } else if IfThenElse::<Self>::peek(input) {
            Self::IfThenElse(input.parse()?)
        } else if Emit::peek(input) {
            Self::Emit(input.parse()?)
        } else if When::peek(input) {
            return Err(input.error("'when' is a root expression"));
        } else {
            Self::parse_prec4(input)?
        };
        Ok(expression)
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust reactive expression kind AST.
pub enum ReactExpr {
    /// Stream expression.
    Expr(Expr),
    /// Pattern matching event.
    When(When),
}
mk_new! { impl ReactExpr =>
    Expr: expr(arg: Expr = arg)
    When: when_match(arg: When = arg)
}
impl Parse for ReactExpr {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let expression = if When::peek(input) {
            Self::when_match(input.parse()?)
        } else {
            Self::expr(input.parse()?)
        };
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
        stream::{Last, Expr, Emit, When, ReactExpr},
        operator::BinaryOperator,
        quote::format_ident,
    }

    #[test]
    fn should_parse_last() {
        let expression: ReactExpr = syn::parse_quote! {last x};
        let control = ReactExpr::expr(Expr::last(Last::new(syn::parse_quote! {x}, None)));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_initialized_last() {
        let expression: ReactExpr = syn::parse_quote! {last x init 0};
        let control = ReactExpr::expr(Expr::last(Last::new(
            syn::parse_quote! {x},
            Some(Expr::cst(Constant::int(syn::parse_quote! {0}))),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_constant() {
        let expression: ReactExpr = syn::parse_quote! {1};
        let control = ReactExpr::expr(Expr::cst(Constant::int(syn::parse_quote! {1})));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_identifier() {
        let expression: ReactExpr = syn::parse_quote! {x};
        let control = ReactExpr::expr(Expr::ident("x"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_application() {
        let expression: ReactExpr = syn::parse_quote! {f(x)};
        let control = ReactExpr::expr(Expr::app(Application::new(
            Expr::ident("f"),
            vec![Expr::ident("x")],
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop() {
        let expression: ReactExpr = syn::parse_quote! {a+b};
        let control = ReactExpr::expr(Expr::binop(Binop::new(
            BinaryOperator::Add,
            Expr::ident("a"),
            Expr::ident("b"),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop_with_precedence() {
        let expression: ReactExpr = syn::parse_quote! {a+b*c};
        let control = ReactExpr::expr(Expr::binop(Binop::new(
            BinaryOperator::Add,
            Expr::ident("a"),
            Expr::Binop(Binop::new(
                BinaryOperator::Mul,
                Expr::ident("b"),
                Expr::ident("c"),
            )),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_typed_abstraction() {
        let expression: ReactExpr = syn::parse_quote! {|x: int| f(x)};
        let control = ReactExpr::expr(Expr::type_abstraction(TypedAbstraction::new(
            vec![("x".into(), Typ::int())],
            Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")])),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_structure() {
        let expression: ReactExpr = syn::parse_quote! {Point {x: 0, y: 1}};
        let control = ReactExpr::expr(Expr::structure(Structure::new(
            "Point",
            vec![
                ("x".into(), Expr::cst(Constant::int(syn::parse_quote! {0}))),
                ("y".into(), Expr::cst(Constant::int(syn::parse_quote! {1}))),
            ],
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple() {
        let expression: ReactExpr = syn::parse_quote! {(x, 0)};
        let control = ReactExpr::expr(Expr::tuple(Tuple::new(vec![
            Expr::ident("x"),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
        ])));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_enumeration() {
        let expression: ReactExpr = syn::parse_quote! {Color::Pink};
        let control = ReactExpr::expr(Expr::enumeration(Enumeration::new("Color", "Pink")));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_array() {
        let expression: ReactExpr = syn::parse_quote! {[1, 2, 3]};
        let control = ReactExpr::expr(Expr::array(Array::new(vec![
            Expr::cst(Constant::int(syn::parse_quote! {1})),
            Expr::cst(Constant::int(syn::parse_quote! {2})),
            Expr::cst(Constant::int(syn::parse_quote! {3})),
        ])));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_match() {
        let expression: ReactExpr = syn::parse_quote! {
            match a {
                Point {x: 0, y: _} => 0,
                Point {x: x, y: _} if f(x) => -1,
                _ => 1,
            }
        };
        let control = ReactExpr::expr(Expr::pat_match(Match::new(
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
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_field_access() {
        let expression: ReactExpr = syn::parse_quote! {p.x};
        let control = ReactExpr::expr(Expr::field_access(FieldAccess::new(Expr::ident("p"), "x")));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple_element_access() {
        let expression: ReactExpr = syn::parse_quote! {t.0};
        let control = ReactExpr::expr(Expr::tuple_access(TupleElementAccess::new(
            Expr::ident("t"),
            0,
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_map() {
        let expression: ReactExpr = syn::parse_quote! {a.map(f)};
        let control = ReactExpr::expr(Expr::map(Map::new(Expr::ident("a"), Expr::ident("f"))));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_fold() {
        let expression: ReactExpr = syn::parse_quote! {a.fold(0, sum)};
        let control = ReactExpr::expr(Expr::fold(Fold::new(
            Expr::ident("a"),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
            Expr::ident("sum"),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_sort() {
        let expression: ReactExpr = syn::parse_quote! {a.sort(order)};
        let control = ReactExpr::expr(Expr::sort(Sort::new(
            Expr::ident("a"),
            Expr::ident("order"),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_zip() {
        let expression: ReactExpr = syn::parse_quote! {zip(a, b, c)};
        let control = ReactExpr::expr(Expr::zip(Zip::new(vec![
            Expr::ident("a"),
            Expr::ident("b"),
            Expr::ident("c"),
        ])));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_emit() {
        let expression: ReactExpr = syn::parse_quote! {emit 0};
        let control = ReactExpr::expr(Expr::emit(Emit::new(
            Default::default(),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
        )));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_when() {
        let expression: ReactExpr = syn::parse_quote! {when let d = p? then emit x};
        let control = ReactExpr::when_match(When::new(
            EventPattern::Let(LetEventPattern::new(
                Default::default(),
                expr::Pattern::ident("d"),
                Default::default(),
                format_ident!("p"),
                Default::default(),
            )),
            None,
            Default::default(),
            Expr::emit(Emit::new(Default::default(), Expr::ident("x"))),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_when_with_guard() {
        let expression: ReactExpr = syn::parse_quote! {when p? if p > 0 then emit x};
        let control = ReactExpr::when_match(When::new(
            EventPattern::Let(LetEventPattern::new(
                Default::default(),
                expr::Pattern::ident("p"),
                Default::default(),
                format_ident!("p"),
                Default::default(),
            )),
            Some(Expr::binop(Binop::new(
                BinaryOperator::Grt,
                Expr::ident("p"),
                Expr::cst(Constant::Integer(syn::parse_quote! {0})),
            ))),
            Default::default(),
            Expr::emit(Emit::new(Default::default(), Expr::ident("x"))),
        ));
        assert_eq!(expression, control)
    }
}
