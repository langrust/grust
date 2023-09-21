use crate::lir::pattern::Pattern;
use crate::lir::statement::r#let::Let;
use crate::lir::statement::Statement as LIRStatement;
use crate::mir::statement::Statement;

use super::expression::lir_from_mir as expression_lir_from_mir;

/// Transform MIR statement into LIR statement.
pub fn lir_from_mir(statement: Statement) -> LIRStatement {
    match statement {
        Statement::Let {
            identifier,
            expression,
        } => LIRStatement::Let(Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier,
            },
            expression: expression_lir_from_mir(expression),
        }),
        Statement::LetTuple {
            identifiers,
            expression,
        } => {
            let elements = identifiers
                .into_iter()
                .map(|identifier| Pattern::Identifier {
                    reference: false,
                    mutable: false,
                    identifier,
                })
                .collect();
            LIRStatement::Let(Let {
                pattern: Pattern::Tuple { elements },
                expression: expression_lir_from_mir(expression),
            })
        }
        Statement::ExpressionLast { expression } => {
            LIRStatement::ExpressionLast(expression_lir_from_mir(expression))
        }
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::constant::Constant;
    use crate::frontend::lir_from_mir::statement::lir_from_mir;
    use crate::lir::expression::{Expression as LIRExpression, FieldExpression};
    use crate::lir::pattern::Pattern;
    use crate::lir::statement::r#let::Let;
    use crate::lir::statement::Statement as LIRStatement;
    use crate::mir::expression::Expression;
    use crate::mir::statement::Statement;

    #[test]
    fn should_create_lir_let_statement_from_mir_let_statement() {
        let statement = Statement::Let {
            identifier: String::from("x"),
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = LIRStatement::Let(Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier: String::from("x"),
            },
            expression: LIRExpression::Literal {
                literal: Constant::Integer(1),
            },
        });
        assert_eq!(lir_from_mir(statement), control)
    }

    #[test]
    fn should_create_lir_let_tuple_statement_from_mir_let_tuple_statement() {
        let statement = Statement::LetTuple {
            identifiers: vec![String::from("o"), String::from("new_node_state")],
            expression: Expression::NodeCall {
                node_identifier: String::from("node_state"),
                input_name: String::from("NodeInput"),
                input_fields: vec![(
                    String::from("i"),
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                )],
            },
        };
        let control = LIRStatement::Let(Let {
            pattern: Pattern::Tuple {
                elements: vec![
                    Pattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("o"),
                    },
                    Pattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("new_node_state"),
                    },
                ],
            },
            expression: LIRExpression::MethodCall {
                receiver: Box::new(LIRExpression::FieldAccess {
                    expression: Box::new(LIRExpression::Identifier {
                        identifier: String::from("self"),
                    }),
                    field: String::from("node_state"),
                }),
                method: String::from("step"),
                arguments: vec![LIRExpression::Structure {
                    name: String::from("NodeInput"),
                    fields: vec![FieldExpression {
                        name: String::from("i"),
                        expression: LIRExpression::Literal {
                            literal: Constant::Integer(1),
                        },
                    }],
                }],
            },
        });
        assert_eq!(lir_from_mir(statement), control)
    }

    #[test]
    fn should_create_lir_last_expression_from_mir_last_expression() {
        let statement = Statement::ExpressionLast {
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = LIRStatement::ExpressionLast(LIRExpression::Literal {
            literal: Constant::Integer(1),
        });
        assert_eq!(lir_from_mir(statement), control)
    }
}
