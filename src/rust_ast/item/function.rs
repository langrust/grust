use crate::rust_ast::{block::Block, item::signature::Signature};

#[derive(Debug, PartialEq, serde::Serialize)]

/// A function definition in Rust.
pub struct Function {
    /// Function's signature.
    pub signature: Signature,
    /// Function's body.
    pub body: Block,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.signature, self.body)
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{constant::Constant, operator::BinaryOperator},
        rust_ast::{
            block::Block,
            expression::Expression,
            item::{function::Function, signature::Signature},
            pattern::Pattern,
            r#type::Type,
            statement::{r#let::Let, Statement},
        },
    };

    #[test]
    fn should_format_function_definition() {
        let function = Function {
            signature: Signature {
                public_visibility: true,
                name: String::from("foo"),
                receiver: None,
                inputs: vec![
                    (
                        String::from("x"),
                        Type::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                    (
                        String::from("y"),
                        Type::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                ],
                output: Type::Identifier {
                    identifier: String::from("i64"),
                },
            },
            body: Block {
                statements: vec![
                    Statement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: true,
                            identifier: String::from("z"),
                        },
                        expression: Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("x"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Identifier {
                                identifier: String::from("y"),
                            }),
                        },
                    }),
                    Statement::ExpressionIntern(Expression::Assignement {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("z"),
                        }),
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("z"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Literal {
                                literal: Constant::Integer(1),
                            }),
                        }),
                    }),
                    Statement::ExpressionLast(Expression::Identifier {
                        identifier: String::from("z"),
                    }),
                ],
            },
        };
        let control = String::from(
            "pub fn foo(x: i64, y: i64) -> i64 { let mut z = x + y; z = z + 1i64; z }",
        );
        assert_eq!(format!("{}", function), control)
    }
}
