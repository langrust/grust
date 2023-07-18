use crate::ast::expression::Expression;
use crate::ir::expression::Expression as IRExpression;

/// Transform AST expressions into IR expressions.
pub fn hir_from_ast(expression: Expression) -> IRExpression {
    match expression {
        Expression::Constant {
            constant,
            typing,
            location,
        } => IRExpression::Constant {
            constant,
            typing: typing.unwrap(),
            location,
        },
        Expression::Call {
            id,
            typing,
            location,
        } => IRExpression::Call {
            id,
            typing: typing.unwrap(),
            location,
        },
        Expression::Application {
            function_expression,
            inputs,
            typing,
            location,
        } => IRExpression::Application {
            function_expression: Box::new(hir_from_ast(*function_expression)),
            inputs: inputs
                .into_iter()
                .map(|input| hir_from_ast(input))
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        Expression::Abstraction {
            inputs,
            expression,
            typing,
            location,
        } => IRExpression::TypedAbstraction {
            inputs: inputs
                .into_iter()
                .zip(typing.clone().unwrap().get_inputs())
                .collect(),
            expression: Box::new(hir_from_ast(*expression)),
            typing: typing.unwrap(),
            location,
        },
        Expression::TypedAbstraction {
            inputs,
            expression,
            typing,
            location,
        } => IRExpression::TypedAbstraction {
            inputs,
            expression: Box::new(hir_from_ast(*expression)),
            typing: typing.unwrap(),
            location,
        },
        Expression::Structure {
            name,
            fields,
            typing,
            location,
        } => IRExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(field, expression)| (field, hir_from_ast(expression)))
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        Expression::Array {
            elements,
            typing,
            location,
        } => IRExpression::Array {
            elements: elements
                .into_iter()
                .map(|expression| hir_from_ast(expression))
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        Expression::Match {
            expression,
            arms,
            typing,
            location,
        } => IRExpression::Match {
            expression: Box::new(hir_from_ast(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, optional_expression, expression)| {
                    (
                        pattern,
                        optional_expression.map(|expression| hir_from_ast(expression)),
                        hir_from_ast(expression),
                    )
                })
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        Expression::When {
            id,
            option,
            present,
            default,
            typing,
            location,
        } => IRExpression::When {
            id,
            option: Box::new(hir_from_ast(*option)),
            present: Box::new(hir_from_ast(*present)),
            default: Box::new(hir_from_ast(*default)),
            typing: typing.unwrap(),
            location,
        },
    }
}

#[cfg(test)]
mod hir_from_ast {
    use crate::ast::expression::Expression;
    use crate::common::{location::Location, type_system::Type};
    use crate::frontend::hir_from_ast::expression::hir_from_ast;
    use crate::ir::expression::Expression as IRExpression;

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = Expression::Call {
            id: String::from("f"),
            typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
            location: Location::default(),
        };
        let hir_expression = hir_from_ast(ast_expression);

        let control = IRExpression::Call {
            id: String::from("f"),
            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
            location: Location::default(),
        };
        assert_eq!(hir_expression, control);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_untyped_ast() {
        let ast_expression = Expression::Call {
            id: String::from("f"),
            typing: None,
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_expression);
    }
}
