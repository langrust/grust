use crate::ast::equation::Equation;
use crate::frontend::hir_from_ast::stream_expression::hir_from_ast as stream_expression_hir_from_ast;
use crate::hir::equation::Equation as HIREquation;

/// Transform AST equations into HIR equations.
pub fn hir_from_ast(equation: Equation) -> HIREquation {
    let Equation {
        scope,
        id,
        signal_type,
        expression,
        location,
    } = equation;

    HIREquation {
        scope,
        id,
        signal_type,
        expression: stream_expression_hir_from_ast(expression),
        location,
    }
}

#[cfg(test)]
mod hir_from_ast {
    use crate::ast::{
        equation::Equation, expression::Expression, stream_expression::StreamExpression,
    };
    use crate::common::{location::Location, scope::Scope, type_system::Type};
    use crate::frontend::hir_from_ast::equation::hir_from_ast;
    use crate::hir::{
        equation::Equation as HIREquation, expression::Expression as HIRExpression,
        stream_expression::StreamExpression as HIRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("i"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_equation = Equation {
            id: String::from("o"),
            scope: Scope::Output,
            signal_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let hir_equation = hir_from_ast(ast_equation);

        let control = HIREquation {
            id: String::from("o"),
            scope: Scope::Output,
            signal_type: Type::Integer,
            expression: HIRStreamExpression::MapApplication {
                function_expression: HIRExpression::Call {
                    id: String::from("f"),
                    typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                    location: Location::default(),
                },
                inputs: vec![HIRStreamExpression::SignalCall {
                    id: String::from("i"),
                    typing: Type::Integer,
                    location: Location::default(),
                }],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        assert_eq!(hir_equation, control);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_untyped_ast() {
        let ast_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("i"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_equation = Equation {
            id: String::from("o"),
            scope: Scope::Output,
            signal_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_equation);
    }
}
