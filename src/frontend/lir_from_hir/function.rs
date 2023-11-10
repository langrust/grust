use crate::{
    ast::function::Function,
    lir::{block::Block, item::function::Function as LIRFunction, statement::Statement},
};

use super::{
    expression::mir_from_hir as expression_mir_from_hir,
    statement::mir_from_hir as statement_mir_from_hir,
};
/// Transform HIR function into LIR function.
pub fn mir_from_hir(function: Function) -> LIRFunction {
    let Function {
        id,
        inputs,
        statements,
        returned: (output, last_expression),
        ..
    } = function;

    let mut statements = statements
        .into_iter()
        .map(statement_mir_from_hir)
        .collect::<Vec<_>>();
    statements.push(Statement::ExpressionLast {
        expression: expression_mir_from_hir(last_expression),
    });

    LIRFunction {
        name: id,
        inputs,
        output,
        body: Block { statements },
    }
}

#[cfg(test)]
mod mir_from_hir {
    use crate::{
        ast::{
            expression::Expression as ASTExpression, function::Function as ASTFunction,
            statement::Statement as ASTStatement,
        },
        common::{location::Location, r#type::Type},
        frontend::lir_from_hir::function::mir_from_hir,
        lir::{
            block::Block, expression::Expression, item::function::Function, statement::Statement,
        },
    };

    #[test]
    fn should_transform_ast_function_definition_into_mir_function_definition() {
        let function = ASTFunction {
            id: format!("add"),
            inputs: vec![(format!("x"), Type::Integer), (format!("y"), Type::Integer)],
            statements: vec![ASTStatement {
                id: format!("o"),
                expression: ASTExpression::Application {
                    function_expression: Box::new(ASTExpression::Call {
                        id: format!(" + "),
                        typing: Some(Type::Abstract(
                            vec![Type::Integer, Type::Integer],
                            Box::new(Type::Integer),
                        )),
                        location: Location::default(),
                    }),
                    inputs: vec![
                        ASTExpression::Call {
                            id: format!("x"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        ASTExpression::Call {
                            id: format!("y"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                    ],
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                element_type: Type::Integer,
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                ASTExpression::Call {
                    id: format!("o"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let control = Function {
            name: format!("add"),
            inputs: vec![(format!("x"), Type::Integer), (format!("y"), Type::Integer)],
            output: Type::Integer,
            body: Block {
                statements: vec![
                    Statement::Let {
                        identifier: format!("o"),
                        expression: Expression::FunctionCall {
                            function: Box::new(Expression::Identifier {
                                identifier: format!(" + "),
                            }),
                            arguments: vec![
                                Expression::Identifier {
                                    identifier: format!("x"),
                                },
                                Expression::Identifier {
                                    identifier: format!("y"),
                                },
                            ],
                        },
                    },
                    Statement::ExpressionLast {
                        expression: Expression::Identifier {
                            identifier: format!("o"),
                        },
                    },
                ],
            },
        };
        assert_eq!(mir_from_hir(function), control)
    }
}
