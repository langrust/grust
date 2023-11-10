use crate::rust_ast::pattern::Pattern;
use crate::rust_ast::statement::r#let::Let;
use crate::rust_ast::statement::Statement as RustASTStatement;
use crate::lir::statement::Statement;

use super::expression::rust_ast_from_lir as expression_rust_ast_from_lir;

/// Transform LIR statement into RustAST statement.
pub fn rust_ast_from_lir(statement: Statement) -> RustASTStatement {
    match statement {
        Statement::Let {
            identifier,
            expression,
        } => RustASTStatement::Let(Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier,
            },
            expression: expression_rust_ast_from_lir(expression),
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
            RustASTStatement::Let(Let {
                pattern: Pattern::Tuple { elements },
                expression: expression_rust_ast_from_lir(expression),
            })
        }
        Statement::ExpressionLast { expression } => {
            RustASTStatement::ExpressionLast(expression_rust_ast_from_lir(expression))
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::common::constant::Constant;
    use crate::frontend::rust_ast_from_lir::statement::rust_ast_from_lir;
    use crate::rust_ast::expression::{Expression as RustASTExpression, FieldExpression};
    use crate::rust_ast::pattern::Pattern;
    use crate::rust_ast::statement::r#let::Let;
    use crate::rust_ast::statement::Statement as RustASTStatement;
    use crate::lir::expression::Expression;
    use crate::lir::statement::Statement;

    #[test]
    fn should_create_rust_ast_let_statement_from_lir_let_statement() {
        let statement = Statement::Let {
            identifier: String::from("x"),
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = RustASTStatement::Let(Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier: String::from("x"),
            },
            expression: RustASTExpression::Literal {
                literal: Constant::Integer(1),
            },
        });
        assert_eq!(rust_ast_from_lir(statement), control)
    }

    #[test]
    fn should_create_rust_ast_let_tuple_statement_from_lir_let_tuple_statement() {
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
        let control = RustASTStatement::Let(Let {
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
            expression: RustASTExpression::MethodCall {
                receiver: Box::new(RustASTExpression::FieldAccess {
                    expression: Box::new(RustASTExpression::Identifier {
                        identifier: String::from("self"),
                    }),
                    field: String::from("node_state"),
                }),
                method: String::from("step"),
                arguments: vec![RustASTExpression::Structure {
                    name: String::from("NodeInput"),
                    fields: vec![FieldExpression {
                        name: String::from("i"),
                        expression: RustASTExpression::Literal {
                            literal: Constant::Integer(1),
                        },
                    }],
                }],
            },
        });
        assert_eq!(rust_ast_from_lir(statement), control)
    }

    #[test]
    fn should_create_rust_ast_last_expression_from_lir_last_expression() {
        let statement = Statement::ExpressionLast {
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = RustASTStatement::ExpressionLast(RustASTExpression::Literal {
            literal: Constant::Integer(1),
        });
        assert_eq!(rust_ast_from_lir(statement), control)
    }
}
