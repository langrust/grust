use crate::ast::statement::Statement;
use crate::frontend::hir_from_ast::expression::hir_from_ast as expression_hir_from_ast;
use crate::ir::statement::Statement as IRStatement;

/// Transform AST statements into IR statements.
pub fn hir_from_ast(statement: Statement) -> IRStatement {
    let Statement {
        id,
        element_type,
        expression,
        location,
    } = statement;

    IRStatement {
        id,
        element_type,
        expression: expression_hir_from_ast(expression),
        location,
    }
}

#[cfg(test)]
mod hir_from_ast {
    use crate::ast::{expression::Expression, statement::Statement};
    use crate::common::{location::Location, type_system::Type};
    use crate::frontend::hir_from_ast::statement::hir_from_ast;
    use crate::ir::{expression::Expression as IRExpression, statement::Statement as IRStatement};

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_statement = Statement {
            id: String::from("y"),
            element_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let hir_statement = hir_from_ast(ast_statement);

        let control = IRStatement {
            id: String::from("y"),
            element_type: Type::Integer,
            expression: IRExpression::Application {
                function_expression: Box::new(IRExpression::Call {
                    id: String::from("f"),
                    typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                    location: Location::default(),
                }),
                inputs: vec![IRExpression::Call {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                }],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        assert_eq!(hir_statement, control);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_untyped_ast() {
        let ast_expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_statement = Statement {
            id: String::from("y"),
            element_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_statement);
    }
}
