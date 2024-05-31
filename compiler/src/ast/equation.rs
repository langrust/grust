use syn::punctuated::Punctuated;
use syn::{braced, Token};
use syn::{parse::Parse, token};

use crate::ast::{
    pattern::Pattern, statement::LetDeclaration, stream_expression::StreamExpression,
};

use super::keyword;

pub struct Instanciation {
    /// Pattern of instanciated signals.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signals.
    pub expression: StreamExpression,
    pub semi_token: Token![;],
}
impl Parse for Instanciation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern: Pattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: StreamExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Instanciation {
            pattern,
            eq_token,
            expression,
            semi_token,
        })
    }
}

/// Arm for matching expression.
pub struct Arm {
    /// The pattern to match.
    pub pattern: Pattern,
    /// The optional guard.
    pub guard: Option<(Token![if], StreamExpression)>,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}
impl Parse for Arm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let token = input.parse()?;
                let guard = input.parse()?;
                Some((token, guard))
            } else {
                None
            }
        };
        let arrow_token = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let equations = {
            let mut equations = Vec::new();
            while !content.is_empty() {
                equations.push(content.parse()?);
            }
            equations
        };
        Ok(Arm {
            pattern,
            guard,
            arrow_token,
            brace_token,
            equations,
        })
    }
}

pub struct Match {
    pub match_token: Token![match],
    /// The stream expression defining the signals.
    pub expression: StreamExpression,
    pub brace_token: token::Brace,
    /// The different matching cases.
    pub arms: Punctuated<Arm, Token![,]>,
}
impl Parse for Match {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_token = input.parse()?;
        let expression = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let arms: Punctuated<Arm, Token![,]> = Punctuated::parse_terminated(&content)?;

        Ok(Match {
            match_token,
            expression,
            brace_token,
            arms,
        })
    }
}

/// ArmWhenEvent for matching event.
pub struct ArmWhenEvent {
    /// The pattern receiving the value of the event.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The optional timeout.
    pub timeout_token: Option<keyword::timeout>,
    /// The event to match.
    pub event: syn::Ident,
    /// The optional guard.
    pub guard: Option<(Token![if], StreamExpression)>,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}
impl Parse for ArmWhenEvent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern = input.parse()?;
        let eq_token = input.parse()?;
        let timeout_token = {
            if input.fork().peek(keyword::timeout) {
                Some(input.parse()?)
            } else {
                None
            }
        };
        let event = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let token = input.parse()?;
                let guard = input.parse()?;
                Some((token, guard))
            } else {
                None
            }
        };
        let arrow_token = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let equations = {
            let mut equations = Vec::new();
            while !content.is_empty() {
                equations.push(content.parse()?);
            }
            equations
        };
        Ok(ArmWhenEvent {
            pattern,
            eq_token,
            timeout_token,
            event,
            guard,
            arrow_token,
            brace_token,
            equations,
        })
    }
}

/// DefaultArmWhen for absence of events.
pub struct DefaultArmWhen {
    pub otherwise_token: keyword::otherwise,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}
impl Parse for DefaultArmWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let otherwise_token = input.parse()?;
        let arrow_token = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let equations = {
            let mut equations = Vec::new();
            while !content.is_empty() {
                equations.push(content.parse()?);
            }
            equations
        };
        Ok(DefaultArmWhen {
            otherwise_token,
            arrow_token,
            brace_token,
            equations,
        })
    }
}

/// ArmWhen for matching expression.
pub enum ArmWhen {
    ArmWhenEvent(ArmWhenEvent),
    Default(DefaultArmWhen),
}
impl Parse for ArmWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::otherwise) {
            Ok(ArmWhen::Default(input.parse()?))
        } else {
            Ok(ArmWhen::ArmWhenEvent(input.parse()?))
        }
    }
}

pub struct MatchWhen {
    pub when_token: keyword::when,
    pub brace_token: token::Brace,
    /// The different matching cases.
    pub arms: Punctuated<ArmWhen, Token![,]>,
}
impl Parse for MatchWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let when_token = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let arms: Punctuated<ArmWhen, Token![,]> = Punctuated::parse_terminated(&content)?;

        Ok(MatchWhen {
            when_token,
            brace_token,
            arms,
        })
    }
}

/// GRust equation AST.
pub enum Equation {
    LocalDef(LetDeclaration<StreamExpression>),
    OutputDef(Instanciation),
    Match(Match),
    MatchWhen(MatchWhen),
}
impl Parse for Equation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![match]) {
            Ok(Equation::Match(input.parse()?))
        } else if input.peek(keyword::when) {
            Ok(Equation::MatchWhen(input.parse()?))
        } else if input.peek(Token![let]) {
            Ok(Equation::LocalDef(input.parse()?))
        } else {
            Ok(Equation::OutputDef(input.parse()?))
        }
    }
}

#[cfg(test)]
mod parse_equation {
    use std::fmt::Debug;

    use crate::{
        ast::{
            expression::{Binop, IfThenElse, Tuple},
            pattern::{Pattern, Tuple as PatTuple, Typed},
            stream_expression::{FollowedBy, StreamExpression},
        },
        common::{constant::Constant, operator::BinaryOperator, r#type::Type},
    };

    use super::Equation;

    impl PartialEq for Equation {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::LocalDef(l0), Self::LocalDef(r0)) => {
                    l0.expression == r0.expression && l0.typed_pattern == r0.typed_pattern
                }
                (Self::OutputDef(l0), Self::OutputDef(r0)) => {
                    l0.expression == r0.expression && l0.pattern == r0.pattern
                }
                _ => false,
            }
        }
    }
    impl Debug for Equation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Equation::LocalDef(arg0) => f
                    .debug_tuple("LocalDef")
                    .field(&arg0.typed_pattern)
                    .field(&arg0.expression)
                    .finish(),
                Equation::OutputDef(arg0) => f
                    .debug_tuple("OutputDef")
                    .field(&arg0.pattern)
                    .field(&arg0.expression)
                    .finish(),
                Equation::Match(arg0) => f
                    .debug_tuple("Match")
                    .field(&arg0.expression)
                    .field(
                        &arg0
                            .arms
                            .iter()
                            .map(|arm| {
                                (
                                    &arm.pattern,
                                    arm.guard.as_ref().map(|(_, expr)| expr),
                                    &arm.equations,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .finish(),
                Equation::MatchWhen(arg0) => f
                    .debug_tuple("MatchWhen")
                    .field(
                        &arg0
                            .arms
                            .iter()
                            .map(|arm| match arm {
                                super::ArmWhen::ArmWhenEvent(arm) => (
                                    Some((
                                        &arm.pattern,
                                        &arm.event,
                                        arm.guard.as_ref().map(|(_, expr)| expr),
                                    )),
                                    &arm.equations,
                                ),
                                super::ArmWhen::Default(arm) => (None, &arm.equations),
                            })
                            .collect::<Vec<_>>(),
                    )
                    .finish(),
            }
        }
    }

    #[test]
    fn should_parse_output_definition() {
        let equation: Equation = syn::parse_quote! {o = if res then 0 else (0 fby o) + inc;};
        let control = Equation::OutputDef(super::Instanciation {
            pattern: syn::parse_quote! {o},
            eq_token: syn::parse_quote! {=},
            expression: StreamExpression::IfThenElse(IfThenElse {
                expression: Box::new(StreamExpression::Identifier(String::from("res"))),
                true_expression: Box::new(StreamExpression::Constant(Constant::Integer(
                    syn::parse_quote! {0},
                ))),
                false_expression: Box::new(StreamExpression::Binop(Binop {
                    op: BinaryOperator::Add,
                    left_expression: Box::new(StreamExpression::FollowedBy(FollowedBy {
                        constant: Box::new(StreamExpression::Constant(Constant::Integer(
                            syn::parse_quote! {0},
                        ))),
                        expression: Box::new(StreamExpression::Identifier(String::from("o"))),
                    })),
                    right_expression: Box::new(StreamExpression::Identifier(String::from("inc"))),
                })),
            }),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_tuple_instanciation() {
        let equation: Equation = syn::parse_quote! {
            (o1, o2) = if res then (0, 0) else ((0 fby o1) + inc1, (0 fby o2) + inc2);
        };
        let control = Equation::OutputDef(super::Instanciation {
            pattern: Pattern::Tuple(PatTuple {
                elements: vec![syn::parse_quote! {o1}, syn::parse_quote! {o2}],
            }),
            eq_token: syn::parse_quote! {=},
            expression: StreamExpression::IfThenElse(IfThenElse {
                expression: Box::new(StreamExpression::Identifier(String::from("res"))),
                true_expression: Box::new(StreamExpression::Tuple(Tuple {
                    elements: vec![
                        StreamExpression::Constant(Constant::Integer(syn::parse_quote! {0})),
                        StreamExpression::Constant(Constant::Integer(syn::parse_quote! {0})),
                    ],
                })),
                false_expression: Box::new(StreamExpression::Tuple(Tuple {
                    elements: vec![
                        StreamExpression::Binop(Binop {
                            op: BinaryOperator::Add,
                            left_expression: Box::new(StreamExpression::FollowedBy(FollowedBy {
                                constant: Box::new(StreamExpression::Constant(Constant::Integer(
                                    syn::parse_quote! {0},
                                ))),
                                expression: Box::new(StreamExpression::Identifier(String::from(
                                    "o1",
                                ))),
                            })),
                            right_expression: Box::new(StreamExpression::Identifier(String::from(
                                "inc1",
                            ))),
                        }),
                        StreamExpression::Binop(Binop {
                            op: BinaryOperator::Add,
                            left_expression: Box::new(StreamExpression::FollowedBy(FollowedBy {
                                constant: Box::new(StreamExpression::Constant(Constant::Integer(
                                    syn::parse_quote! {0},
                                ))),
                                expression: Box::new(StreamExpression::Identifier(String::from(
                                    "o2",
                                ))),
                            })),
                            right_expression: Box::new(StreamExpression::Identifier(String::from(
                                "inc2",
                            ))),
                        }),
                    ],
                })),
            }),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_local_definition() {
        let equation: Equation =
            syn::parse_quote! {let o: int = if res then 0 else (0 fby o) + inc;};
        let control = Equation::LocalDef(super::LetDeclaration {
            let_token: syn::parse_quote!(let),
            typed_pattern: Pattern::Typed(Typed {
                pattern: syn::parse_quote!(o),
                colon_token: syn::parse_quote!(:),
                typing: Type::Integer,
            }),
            eq_token: syn::parse_quote!(=),
            expression: StreamExpression::IfThenElse(IfThenElse {
                expression: Box::new(StreamExpression::Identifier(String::from("res"))),
                true_expression: Box::new(StreamExpression::Constant(Constant::Integer(
                    syn::parse_quote! {0},
                ))),
                false_expression: Box::new(StreamExpression::Binop(Binop {
                    op: BinaryOperator::Add,
                    left_expression: Box::new(StreamExpression::FollowedBy(FollowedBy {
                        constant: Box::new(StreamExpression::Constant(Constant::Integer(
                            syn::parse_quote! {0},
                        ))),
                        expression: Box::new(StreamExpression::Identifier(String::from("o"))),
                    })),
                    right_expression: Box::new(StreamExpression::Identifier(String::from("inc"))),
                })),
            }),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_multiple_definitions() {
        let equation: Equation = syn::parse_quote! {
            let (o1: int, o2: int) = if res then (0, 0) else ((0 fby o1) + inc1, (0 fby o2) + inc2);
        };
        let control = Equation::LocalDef(super::LetDeclaration {
            let_token: syn::parse_quote!(let),
            typed_pattern: Pattern::Tuple(PatTuple {
                elements: vec![
                    Pattern::Typed(Typed {
                        pattern: syn::parse_quote!(o1),
                        colon_token: syn::parse_quote!(:),
                        typing: Type::Integer,
                    }),
                    Pattern::Typed(Typed {
                        pattern: syn::parse_quote!(o2),
                        colon_token: syn::parse_quote!(:),
                        typing: Type::Integer,
                    }),
                ],
            }),
            eq_token: syn::parse_quote!(=),
            expression: StreamExpression::IfThenElse(IfThenElse {
                expression: Box::new(StreamExpression::Identifier(String::from("res"))),
                true_expression: Box::new(StreamExpression::Tuple(Tuple {
                    elements: vec![
                        StreamExpression::Constant(Constant::Integer(syn::parse_quote! {0})),
                        StreamExpression::Constant(Constant::Integer(syn::parse_quote! {0})),
                    ],
                })),
                false_expression: Box::new(StreamExpression::Tuple(Tuple {
                    elements: vec![
                        StreamExpression::Binop(Binop {
                            op: BinaryOperator::Add,
                            left_expression: Box::new(StreamExpression::FollowedBy(FollowedBy {
                                constant: Box::new(StreamExpression::Constant(Constant::Integer(
                                    syn::parse_quote! {0},
                                ))),
                                expression: Box::new(StreamExpression::Identifier(String::from(
                                    "o1",
                                ))),
                            })),
                            right_expression: Box::new(StreamExpression::Identifier(String::from(
                                "inc1",
                            ))),
                        }),
                        StreamExpression::Binop(Binop {
                            op: BinaryOperator::Add,
                            left_expression: Box::new(StreamExpression::FollowedBy(FollowedBy {
                                constant: Box::new(StreamExpression::Constant(Constant::Integer(
                                    syn::parse_quote! {0},
                                ))),
                                expression: Box::new(StreamExpression::Identifier(String::from(
                                    "o2",
                                ))),
                            })),
                            right_expression: Box::new(StreamExpression::Identifier(String::from(
                                "inc2",
                            ))),
                        }),
                    ],
                })),
            }),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }
}
