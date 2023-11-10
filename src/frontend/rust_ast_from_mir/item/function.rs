use crate::frontend::rust_ast_from_mir::{
    block::lir_from_mir as block_lir_from_mir, r#type::lir_from_mir as type_lir_from_mir,
};
use crate::rust_ast::item::{function::Function as RustASTFunction, signature::Signature};
use crate::mir::item::function::Function;

/// Transform MIR function into RustAST function.
pub fn lir_from_mir(function: Function) -> RustASTFunction {
    let inputs = function
        .inputs
        .into_iter()
        .map(|(name, r#type)| (name, type_lir_from_mir(r#type)))
        .collect();
    let signature = Signature {
        public_visibility: true,
        name: function.name,
        receiver: None,
        inputs,
        output: type_lir_from_mir(function.output),
    };
    RustASTFunction {
        signature,
        body: block_lir_from_mir(function.body),
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::frontend::rust_ast_from_mir::item::function::lir_from_mir;
    use crate::rust_ast::block::Block as RustASTBlock;
    use crate::rust_ast::expression::Expression as RustASTExpression;
    use crate::rust_ast::item::function::Function as RustASTFunction;
    use crate::rust_ast::item::signature::Signature;
    use crate::rust_ast::r#type::Type as RustASTType;
    use crate::rust_ast::statement::Statement as RustASTStatement;
    use crate::mir::block::Block;
    use crate::mir::expression::Expression;
    use crate::mir::item::function::Function;
    use crate::mir::statement::Statement;

    #[test]
    fn should_create_lir_function_from_mir_function() {
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
        assert_eq!(lir_from_mir(function), control)
    }
}
