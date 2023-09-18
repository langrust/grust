use crate::lir::statement::Statement;

/// A block of statements.
pub struct Block {
    /// Statements of the block.
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let statements = self
            .statements
            .iter()
            .map(|statement| format!("{statement}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{{ {statements} }}")
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{constant::Constant, operator::BinaryOperator},
        lir::{
            block::Block,
            expression::Expression,
            statement::{r#let::Let, Statement},
        },
    };

    #[test]
    fn should_format_block_expression() {
        let block = Block {
            statements: vec![
                Statement::Let(Let {
                    reference: false,
                    mutable: true,
                    identifiant: String::from("x"),
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                }),
                Statement::ExpressionIntern(Expression::Assignement {
                    left: Box::new(Expression::Identifier {
                        identifier: String::from("x"),
                    }),
                    right: Box::new(Expression::Binary {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("x"),
                        }),
                        operator: BinaryOperator::Add,
                        right: Box::new(Expression::Literal {
                            literal: Constant::Integer(1),
                        }),
                    }),
                }),
                Statement::ExpressionLast(Expression::Identifier {
                    identifier: String::from("x"),
                }),
            ],
        };
        let control = String::from("{ let mut x = 1i64; x = x + 1i64; x }");
        assert_eq!(format!("{}", block), control)
    }
}
