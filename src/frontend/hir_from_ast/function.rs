use crate::ast::function::Function;
use crate::frontend::hir_from_ast::expression::hir_from_ast as expression_hir_from_ast;
use crate::frontend::hir_from_ast::statement::hir_from_ast as statement_hir_from_ast;
use crate::ir::function::Function as IRFunction;

/// Transform AST functions into IR function.
pub fn hir_from_ast(function: Function) -> IRFunction {
    let Function {
        id,
        inputs,
        statements,
        returned: (returned_type, returned_expression),
        location,
    } = function;

    IRFunction {
        id,
        inputs,
        statements: statements
            .into_iter()
            .map(|statement| statement_hir_from_ast(statement))
            .collect(),
        returned: (returned_type, expression_hir_from_ast(returned_expression)),
        location,
    }
}

#[cfg(test)]
mod hir_from_ast {
    use crate::ast::{expression::Expression, function::Function, statement::Statement};
    use crate::common::{location::Location, type_system::Type};
    use crate::frontend::hir_from_ast::function::hir_from_ast;
    use crate::ir::{
        expression::Expression as IRExpression, function::Function as IRFunction,
        statement::Statement as IRStatement,
    };

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
        let ast_returned_expression = Expression::Call {
            id: String::from("y"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![ast_statement],
            returned: (Type::Integer, ast_returned_expression),
            location: Location::default(),
        };
        let hir_function = hir_from_ast(ast_function);

        let control = IRFunction {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![IRStatement {
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
            }],
            returned: (
                Type::Integer,
                IRExpression::Call {
                    id: String::from("y"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        assert_eq!(hir_function, control);
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
        let ast_returned_expression = Expression::Call {
            id: String::from("y"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![ast_statement],
            returned: (Type::Integer, ast_returned_expression),
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_function);
    }
}
