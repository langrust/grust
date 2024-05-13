use syn::parse::Parse;
use syn::Token;

use crate::ast::{statement::LetDeclaration, stream_expression::StreamExpression};

pub struct Instanciation {
    /// Identifier of the signal.
    pub ident: syn::Ident,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    pub semi_token: Token![;],
}
impl Instanciation {
    pub fn get_ident(&self) -> &syn::Ident {
        &self.ident
    }
}
impl Parse for Instanciation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: StreamExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Instanciation {
            ident,
            eq_token,
            expression,
            semi_token,
        })
    }
}

/// GRust equation AST.
pub enum Equation {
    LocalDef(LetDeclaration<StreamExpression>),
    OutputDef(Instanciation),
}
impl Equation {
    pub fn get_ident(&self) -> &syn::Ident {
        match self {
            Equation::LocalDef(declaration) => declaration.get_ident(),
            Equation::OutputDef(instanciation) => instanciation.get_ident(),
        }
    }
    pub fn is_local(&self) -> bool {
        match self {
            Equation::LocalDef(_) => true,
            Equation::OutputDef(_) => false,
        }
    }
}
impl Parse for Equation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let]) {
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
            ident_colon::IdentColon,
            stream_expression::{FollowedBy, StreamExpression},
        },
        common::{constant::Constant, operator::BinaryOperator, r#type::Type},
    };

    use super::Equation;

    impl PartialEq for Equation {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::LocalDef(l0), Self::LocalDef(r0)) => {
                    l0.expression == r0.expression
                        && l0.typed_ident.ident == r0.typed_ident.ident
                        && l0.typed_ident.elem == r0.typed_ident.elem
                }
                (Self::OutputDef(l0), Self::OutputDef(r0)) => {
                    l0.expression == r0.expression && l0.ident == r0.ident
                }
                _ => false,
            }
        }
    }
    impl Debug for Equation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::LocalDef(arg0) => f
                    .debug_tuple("LocalDef")
                    .field(&arg0.typed_ident.ident)
                    .field(&arg0.typed_ident.elem)
                    .field(&arg0.expression)
                    .finish(),
                Self::OutputDef(arg0) => f
                    .debug_tuple("OutputDef")
                    .field(&arg0.ident)
                    .field(&arg0.expression)
                    .finish(),
            }
        }
    }

    #[test]
    fn should_parse_output_definition() {
        let equation: Equation = syn::parse_quote! {o = if res then 0 else (0 fby o) + inc;};
        let control = Equation::OutputDef(super::Instanciation {
            ident: syn::parse_quote! {o},
            eq_token: syn::parse_quote! {=},
            expression: StreamExpression::IfThenElse(IfThenElse {
                expression: Box::new(StreamExpression::Identifier(String::from("res"))),
                true_expression: Box::new(StreamExpression::Constant(Constant::Integer(
                    syn::parse_quote! {0},
                ))),
                false_expression: Box::new(StreamExpression::Binop(Binop {
                    op: BinaryOperator::Add,
                    left_expression: Box::new(StreamExpression::Tuple(Tuple {
                        elements: vec![StreamExpression::FollowedBy(FollowedBy {
                            constant: Box::new(StreamExpression::Constant(Constant::Integer(
                                syn::parse_quote! {0},
                            ))),
                            expression: Box::new(StreamExpression::Identifier(String::from("o"))),
                        })],
                    })),
                    right_expression: Box::new(StreamExpression::Identifier(String::from("inc"))),
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
            typed_ident: IdentColon {
                ident: syn::parse_quote!(o),
                colon: syn::parse_quote!(:),
                elem: Type::Integer,
            },
            eq_token: syn::parse_quote!(=),
            expression: StreamExpression::IfThenElse(IfThenElse {
                expression: Box::new(StreamExpression::Identifier(String::from("res"))),
                true_expression: Box::new(StreamExpression::Constant(Constant::Integer(
                    syn::parse_quote! {0},
                ))),
                false_expression: Box::new(StreamExpression::Binop(Binop {
                    op: BinaryOperator::Add,
                    left_expression: Box::new(StreamExpression::Tuple(Tuple {
                        elements: vec![StreamExpression::FollowedBy(FollowedBy {
                            constant: Box::new(StreamExpression::Constant(Constant::Integer(
                                syn::parse_quote! {0},
                            ))),
                            expression: Box::new(StreamExpression::Identifier(String::from("o"))),
                        })],
                    })),
                    right_expression: Box::new(StreamExpression::Identifier(String::from("inc"))),
                })),
            }),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }
}
