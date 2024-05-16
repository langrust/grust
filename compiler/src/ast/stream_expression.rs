use syn::parse::Parse;

use crate::common::constant::Constant;
use crate::{
    ast::{
        expression::{
            Application, Array, Enumeration, FieldAccess, Fold, Map, Match, Sort, Structure, Tuple,
            TupleElementAccess, TypedAbstraction, Zip,
        },
        keyword,
    },
    common::operator::BinaryOperator,
};

use super::expression::{Binop, IfThenElse, ParsePrec, Unop};

/// Initialized buffer stream expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FollowedBy {
    /// The initialization constant.
    pub constant: Box<StreamExpression>,
    /// The buffered expression.
    pub expression: Box<StreamExpression>,
}
impl FollowedBy {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::fby)
    }
    pub fn parse(
        constant: Box<StreamExpression>,
        input: syn::parse::ParseStream,
    ) -> syn::Result<Self> {
        let _: keyword::fby = input.parse()?;
        let expression = Box::new(input.parse()?);
        Ok(FollowedBy {
            constant,
            expression,
        })
    }
}
impl Parse for FollowedBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let constant = Box::new(input.parse()?);
        let _: keyword::fby = input.parse()?;
        let expression = Box::new(input.parse()?);
        Ok(FollowedBy {
            constant,
            expression,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust stream expression kind AST.
pub enum StreamExpression {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(String),
    /// Application expression.
    Application(Application<StreamExpression>),
    /// Unop expression.
    Unop(Unop<StreamExpression>),
    /// Binop expression.
    Binop(Binop<StreamExpression>),
    /// IfThenElse expression.
    IfThenElse(IfThenElse<StreamExpression>),
    /// Abstraction expression with inputs types.
    TypedAbstraction(TypedAbstraction<StreamExpression>),
    /// Structure expression.
    Structure(Structure<StreamExpression>),
    /// Tuple expression.
    Tuple(Tuple<StreamExpression>),
    /// Enumeration expression.
    Enumeration(Enumeration),
    /// Array expression.
    Array(Array<StreamExpression>),
    /// Pattern matching expression.
    Match(Match<StreamExpression>),
    /// Field access expression.
    FieldAccess(FieldAccess<StreamExpression>),
    /// Tuple element access expression.
    TupleElementAccess(TupleElementAccess<StreamExpression>),
    /// Array map operator expression.
    Map(Map<StreamExpression>),
    /// Array fold operator expression.
    Fold(Fold<StreamExpression>),
    /// Array sort operator expression.
    Sort(Sort<StreamExpression>),
    /// Arrays zip operator expression.
    Zip(Zip<StreamExpression>),
    /// Initialized buffer stream expression.
    FollowedBy(FollowedBy),
}
impl ParsePrec for StreamExpression {
    fn parse_term(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = if input.fork().call(Constant::parse).is_ok() {
            StreamExpression::Constant(input.parse()?)
        } else if Unop::<StreamExpression>::peek(input) {
            StreamExpression::Unop(input.parse()?)
        } else if Zip::<StreamExpression>::peek(input) {
            StreamExpression::Zip(input.parse()?)
        } else if Match::<StreamExpression>::peek(input) {
            StreamExpression::Match(input.parse()?)
        } else if Tuple::<StreamExpression>::peek(input) {
            let mut tuple: Tuple<StreamExpression> = input.parse()?;
            if tuple.elements.len() == 1 {
                tuple.elements.pop().unwrap()
            } else {
                StreamExpression::Tuple(tuple)
            }
        } else if Array::<StreamExpression>::peek(input) {
            StreamExpression::Array(input.parse()?)
        } else if Structure::<StreamExpression>::peek(input) {
            StreamExpression::Structure(input.parse()?)
        } else if Enumeration::peek(input) {
            StreamExpression::Enumeration(input.parse()?)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            StreamExpression::Identifier(ident.to_string())
        } else {
            return Err(input.error("expected expression"));
        };
        loop {
            if Sort::<StreamExpression>::peek(input) {
                expression = StreamExpression::Sort(Sort::<StreamExpression>::parse(
                    Box::new(expression),
                    input,
                )?);
            } else if Map::<StreamExpression>::peek(input) {
                expression = StreamExpression::Map(Map::<StreamExpression>::parse(
                    Box::new(expression),
                    input,
                )?)
            } else if Fold::<StreamExpression>::peek(input) {
                expression = StreamExpression::Fold(Fold::<StreamExpression>::parse(
                    Box::new(expression),
                    input,
                )?)
            } else if TupleElementAccess::<StreamExpression>::peek(input) {
                expression =
                    StreamExpression::TupleElementAccess(
                        TupleElementAccess::<StreamExpression>::parse(Box::new(expression), input)?,
                    )
            } else if FieldAccess::<StreamExpression>::peek(input) {
                expression = StreamExpression::FieldAccess(FieldAccess::<StreamExpression>::parse(
                    Box::new(expression),
                    input,
                )?)
            } else if Application::<StreamExpression>::peek(input) {
                expression = StreamExpression::Application(Application::<StreamExpression>::parse(
                    Box::new(expression),
                    input,
                )?)
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn parse_prec1(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = StreamExpression::parse_term(input)?;

        loop {
            if BinaryOperator::peek_prec1(input) {
                expression = StreamExpression::Binop(Binop::<StreamExpression>::parse_term(
                    Box::new(expression),
                    input,
                )?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec2(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = StreamExpression::parse_prec1(input)?;

        loop {
            if BinaryOperator::peek_prec2(input) {
                expression = StreamExpression::Binop(Binop::<StreamExpression>::parse_prec1(
                    Box::new(expression),
                    input,
                )?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec3(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = StreamExpression::parse_prec2(input)?;

        loop {
            if BinaryOperator::peek_prec3(input) {
                expression = StreamExpression::Binop(Binop::<StreamExpression>::parse_prec2(
                    Box::new(expression),
                    input,
                )?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec4(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = StreamExpression::parse_prec3(input)?;

        loop {
            if BinaryOperator::peek_prec4(input) {
                expression = StreamExpression::Binop(Binop::<StreamExpression>::parse_prec3(
                    Box::new(expression),
                    input,
                )?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
}
impl Parse for StreamExpression {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = if TypedAbstraction::<StreamExpression>::peek(input) {
            StreamExpression::TypedAbstraction(input.parse()?)
        } else if IfThenElse::<StreamExpression>::peek(input) {
            StreamExpression::IfThenElse(input.parse()?)
        } else {
            StreamExpression::parse_prec4(input)?
        };
        loop {
            if FollowedBy::peek(input) {
                expression =
                    StreamExpression::FollowedBy(FollowedBy::parse(Box::new(expression), input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
}

#[cfg(test)]
mod parse_stream_expression {
    use crate::{
        ast::{
            expression::{
                Application, Arm, Array, Binop, Enumeration, FieldAccess, Fold, Map, Match, Sort,
                Structure, Tuple, TupleElementAccess, TypedAbstraction, Zip,
            },
            pattern::{self, Pattern},
            stream_expression::{FollowedBy, StreamExpression},
        },
        common::{constant::Constant, operator::BinaryOperator, r#type::Type},
    };

    #[test]
    fn should_parse_followed_by() {
        let expression: StreamExpression = syn::parse_quote! {0 fby p.x};
        let control = StreamExpression::FollowedBy(FollowedBy {
            constant: Box::new(StreamExpression::Constant(Constant::Integer(
                syn::parse_quote! {0},
            ))),
            expression: Box::new(StreamExpression::FieldAccess(FieldAccess {
                expression: Box::new(StreamExpression::Identifier(String::from("p"))),
                field: String::from("x"),
            })),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_constant() {
        let expression: StreamExpression = syn::parse_quote! {1};
        let control = StreamExpression::Constant(Constant::Integer(syn::parse_quote! {1}));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_identifier() {
        let expression: StreamExpression = syn::parse_quote! {x};
        let control = StreamExpression::Identifier(String::from("x"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_application() {
        let expression: StreamExpression = syn::parse_quote! {f(x)};
        let control = StreamExpression::Application(Application {
            function_expression: Box::new(StreamExpression::Identifier(String::from("f"))),
            inputs: vec![StreamExpression::Identifier(String::from("x"))],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop() {
        let expression: StreamExpression = syn::parse_quote! {a+b};
        let control = StreamExpression::Binop(Binop {
            op: BinaryOperator::Add,
            left_expression: Box::new(StreamExpression::Identifier(String::from("a"))),
            right_expression: Box::new(StreamExpression::Identifier(String::from("b"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop_with_precedence() {
        let expression: StreamExpression = syn::parse_quote! {a+b*c};
        let control = StreamExpression::Binop(Binop {
            op: BinaryOperator::Add,
            left_expression: Box::new(StreamExpression::Identifier(String::from("a"))),
            right_expression: Box::new(StreamExpression::Binop(Binop {
                op: BinaryOperator::Mul,
                left_expression: Box::new(StreamExpression::Identifier(String::from("b"))),
                right_expression: Box::new(StreamExpression::Identifier(String::from("c"))),
            })),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_typed_abstraction() {
        let expression: StreamExpression = syn::parse_quote! {|x: int| f(x)};
        let control = StreamExpression::TypedAbstraction(TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(StreamExpression::Application(Application {
                function_expression: Box::new(StreamExpression::Identifier(String::from("f"))),
                inputs: vec![StreamExpression::Identifier(String::from("x"))],
            })),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_structure() {
        let expression: StreamExpression = syn::parse_quote! {Point {x: 0, y: 1}};
        let control = StreamExpression::Structure(Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant(Constant::Integer(syn::parse_quote! {0})),
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant(Constant::Integer(syn::parse_quote! {1})),
                ),
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple() {
        let expression: StreamExpression = syn::parse_quote! {(x, 0)};
        let control = StreamExpression::Tuple(Tuple {
            elements: vec![
                StreamExpression::Identifier(String::from("x")),
                StreamExpression::Constant(Constant::Integer(syn::parse_quote! {0})),
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_enumeration() {
        let expression: StreamExpression = syn::parse_quote! {Color::Pink};
        let control = StreamExpression::Enumeration(Enumeration {
            enum_name: String::from("Color"),
            elem_name: String::from("Pink"),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_array() {
        let expression: StreamExpression = syn::parse_quote! {[1, 2, 3]};
        let control = StreamExpression::Array(Array {
            elements: vec![
                StreamExpression::Constant(Constant::Integer(syn::parse_quote! {1})),
                StreamExpression::Constant(Constant::Integer(syn::parse_quote! {2})),
                StreamExpression::Constant(Constant::Integer(syn::parse_quote! {3})),
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_match() {
        let expression: StreamExpression = syn::parse_quote! {
            match a {
                Point {x: 0, y: _} => 0,
                Point {x: x, y: _} if f(x) => -1,
                _ => 1,
            }
        };
        let control = StreamExpression::Match(Match {
            expression: Box::new(StreamExpression::Identifier(String::from("a"))),
            arms: vec![
                Arm {
                    pattern: Pattern::Structure(pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Some(Pattern::Constant(Constant::Integer(syn::parse_quote! {0}))),
                            ),
                            (String::from("y"), Some(Pattern::Default)),
                        ],
                        rest: None,
                    }),
                    guard: None,
                    expression: StreamExpression::Constant(Constant::Integer(
                        syn::parse_quote! {0},
                    )),
                },
                Arm {
                    pattern: Pattern::Structure(pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Some(Pattern::Identifier(String::from("x"))),
                            ),
                            (String::from("y"), Some(Pattern::Default)),
                        ],
                        rest: None,
                    }),
                    guard: Some(StreamExpression::Application(Application {
                        function_expression: Box::new(StreamExpression::Identifier(String::from(
                            "f",
                        ))),
                        inputs: vec![StreamExpression::Identifier(String::from("x"))],
                    })),
                    expression: StreamExpression::Constant(Constant::Integer(
                        syn::parse_quote! {-1},
                    )),
                },
                Arm {
                    pattern: Pattern::Default,
                    guard: None,
                    expression: StreamExpression::Constant(Constant::Integer(
                        syn::parse_quote! {1},
                    )),
                },
            ],
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_field_access() {
        let expression: StreamExpression = syn::parse_quote! {p.x};
        let control = StreamExpression::FieldAccess(FieldAccess {
            expression: Box::new(StreamExpression::Identifier(String::from("p"))),
            field: String::from("x"),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple_element_access() {
        let expression: StreamExpression = syn::parse_quote! {t.0};
        let control = StreamExpression::TupleElementAccess(TupleElementAccess {
            expression: Box::new(StreamExpression::Identifier(String::from("t"))),
            element_number: 0,
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_map() {
        let expression: StreamExpression = syn::parse_quote! {a.map(f)};
        let control = StreamExpression::Map(Map {
            expression: Box::new(StreamExpression::Identifier(String::from("a"))),
            function_expression: Box::new(StreamExpression::Identifier(String::from("f"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_fold() {
        let expression: StreamExpression = syn::parse_quote! {a.fold(0, sum)};
        let control = StreamExpression::Fold(Fold {
            expression: Box::new(StreamExpression::Identifier(String::from("a"))),
            initialization_expression: Box::new(StreamExpression::Constant(Constant::Integer(
                syn::parse_quote! {0},
            ))),
            function_expression: Box::new(StreamExpression::Identifier(String::from("sum"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_sort() {
        let expression: StreamExpression = syn::parse_quote! {a.sort(order)};
        let control = StreamExpression::Sort(Sort {
            expression: Box::new(StreamExpression::Identifier(String::from("a"))),
            function_expression: Box::new(StreamExpression::Identifier(String::from("order"))),
        });
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_zip() {
        let expression: StreamExpression = syn::parse_quote! {zip(a, b, c)};
        let control = StreamExpression::Zip(Zip {
            arrays: vec![
                StreamExpression::Identifier(String::from("a")),
                StreamExpression::Identifier(String::from("b")),
                StreamExpression::Identifier(String::from("c")),
            ],
        });
        assert_eq!(expression, control)
    }
}
