use crate::{hir::equation::Equation, lir::statement::Statement};

use super::stream_expression::lir_from_hir as stream_expression_lir_from_hir;

/// Transform HIR equation into LIR statement.
pub fn lir_from_hir(equation: Equation) -> Statement {
    let Equation { id, expression, .. } = equation;
    Statement::Let {
        identifier: id,
        expression: stream_expression_lir_from_hir(expression),
    }
}

#[cfg(test)]
mod lir_from_hir {
    use crate::{
        common::{constant::Constant, location::Location, r#type::Type, scope::Scope},
        frontend::lir_from_hir::equation::lir_from_hir,
        hir::{
            dependencies::Dependencies, equation::Equation, signal::Signal,
            stream_expression::StreamExpression,
        },
        lir::{expression::Expression, statement::Statement},
    };

    #[test]
    fn should_transform_hir_equation_of_constant_into_lir_let_statement() {
        let equation = Equation {
            scope: Scope::Local,
            id: format!("y"),
            expression: StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };
        let control = Statement::Let {
            identifier: format!("y"),
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        assert_eq!(lir_from_hir(equation), control)
    }

    #[test]
    fn should_transform_hir_equation_of_unitary_node_application_into_lir_tuple_statement() {
        let equation = Equation {
            scope: Scope::Local,
            id: format!("y"),
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("my_node_o_y")),
                node: format!("my_node"),
                signal: format!("o"),
                inputs: vec![
                    (
                        format!("i"),
                        StreamExpression::SignalCall {
                            signal: Signal {
                                id: format!("x"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                        },
                    ),
                    (
                        format!("j"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                    ),
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("x"), 0)]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };
        let control = Statement::Let {
            identifier: format!("y"),
            expression: Expression::NodeCall {
                node_identifier: format!("my_node_o_y"),
                input_name: format!("MyNodeOInput"),
                input_fields: vec![
                    (
                        format!("i"),
                        Expression::Identifier {
                            identifier: format!("x"),
                        },
                    ),
                    (
                        format!("j"),
                        Expression::Literal {
                            literal: Constant::Integer(1),
                        },
                    ),
                ],
            },
        };
        assert_eq!(lir_from_hir(equation), control)
    }
}
