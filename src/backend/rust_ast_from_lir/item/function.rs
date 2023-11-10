use crate::backend::rust_ast_from_lir::{
    block::rust_ast_from_lir as block_rust_ast_from_lir, r#type::rust_ast_from_lir as type_rust_ast_from_lir,
};
use crate::rust_ast::item::{function::Function as RustASTFunction, signature::Signature};
use crate::lir::item::function::Function;

/// Transform LIR function into RustAST function.
pub fn rust_ast_from_lir(function: Function) -> RustASTFunction {
    let inputs = function
        .inputs
        .into_iter()
        .map(|(name, r#type)| (name, type_rust_ast_from_lir(r#type)))
        .collect();
    let signature = Signature {
        public_visibility: true,
        name: function.name,
        receiver: None,
        inputs,
        output: type_rust_ast_from_lir(function.output),
    };
    RustASTFunction {
        signature,
        body: block_rust_ast_from_lir(function.body),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::backend::rust_ast_from_lir::item::function::rust_ast_from_lir;
    use crate::rust_ast::block::Block as RustASTBlock;
    use crate::rust_ast::expression::Expression as RustASTExpression;
    use crate::rust_ast::item::function::Function as RustASTFunction;
    use crate::rust_ast::item::signature::Signature;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::rust_ast::statement::Statement as RustASTStatement;
    use crate::lir::block::Block;
    use crate::lir::expression::Expression;
    use crate::lir::item::function::Function;
    use crate::lir::statement::Statement;

    #[test]
    fn should_create_rust_ast_function_from_lir_function() {
        let function = Function {
            name: String::from("foo"),
            inputs: vec![
                (String::from("a"), Type::Integer),
                (String::from("b"), Type::Integer),
            ],
            output: Type::Integer,
            body: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::FunctionCall {
                        function: Box::new(Expression::Identifier {
                            identifier: String::from(" + "),
                        }),
                        arguments: vec![
                            Expression::Identifier {
                                identifier: String::from("a"),
                            },
                            Expression::Identifier {
                                identifier: String::from("b"),
                            },
                        ],
                    },
                }],
            },
        };
        let control = RustASTFunction {
            signature: Signature {
                public_visibility: true,
                name: String::from("foo"),
                receiver: None,
                inputs: vec![
                    (
                        String::from("a"),
                        RustASTType::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                    (
                        String::from("b"),
                        RustASTType::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                ],
                output: RustASTType::Identifier {
                    identifier: String::from("i64"),
                },
            },
            body: RustASTBlock {
                statements: vec![RustASTStatement::ExpressionLast(RustASTExpression::Binary {
                    left: Box::new(RustASTExpression::Identifier {
                        identifier: String::from("a"),
                    }),
                    operator: BinaryOperator::Add,
                    right: Box::new(RustASTExpression::Identifier {
                        identifier: String::from("b"),
                    }),
                })],
            },
        };
        assert_eq!(rust_ast_from_lir(function), control)
    }
}
