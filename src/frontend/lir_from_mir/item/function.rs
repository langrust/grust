use crate::frontend::lir_from_mir::{
    block::lir_from_mir as block_lir_from_mir, r#type::lir_from_mir as type_lir_from_mir,
};
use crate::lir::item::{function::Function as LIRFunction, signature::Signature};
use crate::mir::item::function::Function;

/// Transform MIR function into LIR function.
pub fn lir_from_mir(function: Function) -> LIRFunction {
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
    LIRFunction {
        signature,
        body: block_lir_from_mir(function.body),
    }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::operator::BinaryOperator;
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::item::function::lir_from_mir;
    use crate::lir::block::Block as LIRBlock;
    use crate::lir::expression::Expression as LIRExpression;
    use crate::lir::item::function::Function as LIRFunction;
    use crate::lir::item::signature::Signature;
    use crate::lir::r#type::Type as LIRType;
    use crate::lir::statement::Statement as LIRStatement;
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
        let control = LIRFunction {
            signature: Signature {
                public_visibility: true,
                name: String::from("foo"),
                receiver: None,
                inputs: vec![
                    (
                        String::from("a"),
                        LIRType::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                    (
                        String::from("b"),
                        LIRType::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                ],
                output: LIRType::Identifier {
                    identifier: String::from("i64"),
                },
            },
            body: LIRBlock {
                statements: vec![LIRStatement::ExpressionLast(LIRExpression::Binary {
                    left: Box::new(LIRExpression::Identifier {
                        identifier: String::from("a"),
                    }),
                    operator: BinaryOperator::Add,
                    right: Box::new(LIRExpression::Identifier {
                        identifier: String::from("b"),
                    }),
                })],
            },
        };
        assert_eq!(lir_from_mir(function), control)
    }
}
