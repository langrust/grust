use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    ast::{expression::Expression, pattern::Pattern},
    common::{
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        r#type::Type,
    },
    lir::{
        block::Block, expression::Expression as LIRExpression, item::node_file::import::Import,
        statement::Statement,
    },
};

/// Transform HIR expression into LIR expression.
pub fn lir_from_hir(expression: Expression) -> LIRExpression {
    match expression {
        Expression::Constant { constant, .. } => LIRExpression::Literal { literal: constant },
        Expression::Call { id, .. } => LIRExpression::Identifier { identifier: id },
        Expression::Application {
            function_expression,
            mut inputs,
            ..
        } => match *function_expression {
            Expression::Call { id, .. } if OtherOperator::IfThenElse.to_string() == id => {
                assert!(inputs.len() == 3);
                let else_branch = lir_from_hir(inputs.pop().unwrap());
                let then_branch = lir_from_hir(inputs.pop().unwrap());
                let condition = lir_from_hir(inputs.pop().unwrap());
                LIRExpression::IfThenElse {
                    condition: Box::new(condition),
                    then_branch: Block {
                        statements: vec![Statement::ExpressionLast {
                            expression: then_branch,
                        }],
                    },
                    else_branch: Block {
                        statements: vec![Statement::ExpressionLast {
                            expression: else_branch,
                        }],
                    },
                }
            }
            _ => {
                let arguments = inputs.into_iter().map(lir_from_hir).collect();
                LIRExpression::FunctionCall {
                    function: Box::new(lir_from_hir(*function_expression)),
                    arguments,
                }
            }
        },
        Expression::TypedAbstraction {
            inputs,
            expression,
            typing: Some(Type::Abstract(_, output_type)),
            ..
        } => LIRExpression::Lambda {
            inputs,
            output: *output_type,
            body: Box::new(lir_from_hir(*expression)),
        },
        Expression::Structure { name, fields, .. } => LIRExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(id, expression)| (id, lir_from_hir(expression)))
                .collect(),
        },
        Expression::Array { elements, .. } => LIRExpression::Array {
            elements: elements.into_iter().map(lir_from_hir).collect(),
        },
        Expression::Match {
            expression, arms, ..
        } => LIRExpression::Match {
            matched: Box::new(lir_from_hir(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, expression)| {
                    (pattern, guard.map(lir_from_hir), lir_from_hir(expression))
                })
                .collect(),
        },
        Expression::When {
            id,
            option,
            present,
            default,
            location,
            ..
        } => LIRExpression::Match {
            matched: Box::new(lir_from_hir(*option)),
            arms: vec![
                (
                    Pattern::Some {
                        pattern: Box::new(Pattern::Identifier {
                            name: id,
                            location: location.clone(),
                        }),
                        location: location.clone(),
                    },
                    None,
                    lir_from_hir(*present),
                ),
                (Pattern::None { location }, None, lir_from_hir(*default)),
            ],
        },
        Expression::FieldAccess {
            expression, field, ..
        } => LIRExpression::FieldAccess {
            expression: Box::new(lir_from_hir(*expression)),
            field,
        },
        Expression::TypedAbstraction { .. } | Expression::Abstraction { .. } => {
            unreachable!()
        }
        Expression::Map {
            expression,
            function_expression,
            ..
        } => LIRExpression::Map {
            mapped: Box::new(lir_from_hir(*expression)),
            function: Box::new(lir_from_hir(*function_expression)),
        },
        Expression::Fold {
            expression,
            initialization_expression,
            function_expression,
            ..
        } => LIRExpression::Fold {
            folded: Box::new(lir_from_hir(*expression)),
            initialization: Box::new(lir_from_hir(*initialization_expression)),
            function: Box::new(lir_from_hir(*function_expression)),
        },
        Expression::Sort {
            expression,
            function_expression,
            ..
        } => LIRExpression::Sort {
            sorted: Box::new(lir_from_hir(*expression)),
            function: Box::new(lir_from_hir(*function_expression)),
        },
        Expression::Zip { arrays, .. } => LIRExpression::Zip {
            arrays: arrays.into_iter().map(lir_from_hir).collect(),
        },
    }
}

impl Expression {
    /// Get imports induced by expression.
    pub fn get_imports(&self) -> Vec<Import> {
        match self {
            Expression::Call { .. } => vec![],
            Expression::Constant { constant, .. } => constant.get_imports(),
            Expression::Application {
                function_expression,
                inputs,
                ..
            } => {
                let mut imports = inputs
                    .iter()
                    .flat_map(Expression::get_imports)
                    .collect::<Vec<_>>();
                let mut function_import = match function_expression.as_ref() {
                    Expression::Call { id, .. }
                        if !(BinaryOperator::iter().any(|binary| &binary.to_string() == id)
                            || UnaryOperator::iter().any(|unary| &unary.to_string() == id)
                            || OtherOperator::iter().any(|other| &other.to_string() == id)) =>
                    {
                        vec![Import::Function(id.clone())]
                    }
                    _ => {
                        vec![]
                    }
                };
                imports.append(&mut function_import);
                imports.into_iter().unique().collect()
            }
            Expression::Structure { name, fields, .. } => {
                let mut imports = fields
                    .iter()
                    .flat_map(|(_, expression)| expression.get_imports())
                    .collect::<Vec<_>>();
                imports.push(Import::Structure(name.clone()));
                imports.into_iter().unique().collect()
            }
            Expression::Array { elements, .. } => elements
                .iter()
                .flat_map(Expression::get_imports)
                .unique()
                .collect(),
            Expression::Match {
                expression, arms, ..
            } => {
                let mut arms_imports = arms
                    .iter()
                    .flat_map(|(pattern, guard, expression)| {
                        let mut pattern_imports = pattern.get_imports();
                        let mut guard_imports =
                            guard.as_ref().map_or(vec![], Expression::get_imports);
                        let mut expression_imports = expression.get_imports();

                        let mut imports = vec![];
                        imports.append(&mut pattern_imports);
                        imports.append(&mut guard_imports);
                        imports.append(&mut expression_imports);
                        imports
                    })
                    .collect();
                let mut expression_imports = expression.get_imports();

                let mut imports = vec![];
                imports.append(&mut arms_imports);
                imports.append(&mut expression_imports);
                imports.into_iter().unique().collect()
            }
            Expression::When {
                option,
                present,
                default,
                ..
            } => {
                let mut option_imports = option.get_imports();
                let mut present_imports = present.get_imports();
                let mut default_imports = default.get_imports();

                let mut imports = vec![];
                imports.append(&mut option_imports);
                imports.append(&mut present_imports);
                imports.append(&mut default_imports);
                imports.into_iter().unique().collect()
            }
            Expression::TypedAbstraction { expression, .. } => expression.get_imports(),
            Expression::Abstraction { .. } => unreachable!(),
            Expression::FieldAccess { expression, .. } => expression.get_imports(),
            Expression::Map {
                expression,
                function_expression,
                ..
            } => {
                let mut expression_imports = expression.get_imports();
                let mut function_expression_imports = function_expression.get_imports();

                let mut imports = vec![];
                imports.append(&mut expression_imports);
                imports.append(&mut function_expression_imports);
                imports.into_iter().unique().collect()
            }
            Expression::Fold {
                expression,
                initialization_expression,
                function_expression,
                ..
            } => {
                let mut initialization_expression_imports = initialization_expression.get_imports();
                let mut expression_imports = expression.get_imports();
                let mut function_expression_imports = function_expression.get_imports();

                let mut imports = vec![];
                imports.append(&mut expression_imports);
                imports.append(&mut initialization_expression_imports);
                imports.append(&mut function_expression_imports);
                imports.into_iter().unique().collect()
            }
            Expression::Sort {
                expression,
                function_expression,
                ..
            } => {
                let mut expression_imports = expression.get_imports();
                let mut function_expression_imports = function_expression.get_imports();

                let mut imports = vec![];
                imports.append(&mut expression_imports);
                imports.append(&mut function_expression_imports);
                imports.into_iter().unique().collect()
            }
            Expression::Zip { arrays, .. } => arrays
                .iter()
                .flat_map(Expression::get_imports)
                .unique()
                .collect(),
        }
    }
}

#[cfg(test)]
mod lir_from_hir {
    use crate::{
        ast::{expression::Expression as ASTExpression, pattern::Pattern},
        common::{constant::Constant, location::Location, operator::OtherOperator, r#type::Type},
        frontend::lir_from_hir::expression::lir_from_hir,
        lir::{block::Block, expression::Expression, statement::Statement},
    };

    #[test]
    fn should_transform_ast_constant_into_lir_literal() {
        let expression = ASTExpression::Constant {
            constant: Constant::Integer(1),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Literal {
            literal: Constant::Integer(1),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_call_into_lir_identifier() {
        let expression = ASTExpression::Call {
            id: format!("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Identifier {
            identifier: format!("x"),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_application_into_lir_function_call() {
        let expression = ASTExpression::Application {
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
                ASTExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::FunctionCall {
            function: Box::new(Expression::Identifier {
                identifier: format!(" + "),
            }),
            arguments: vec![
                Expression::Identifier {
                    identifier: format!("x"),
                },
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
            ],
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_application_of_if_then_else_into_lir_if_then_else() {
        let expression = ASTExpression::Application {
            function_expression: Box::new(ASTExpression::Call {
                id: OtherOperator::IfThenElse.to_string(),
                typing: Some(Type::Abstract(
                    vec![Type::Boolean, Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            }),
            inputs: vec![
                ASTExpression::Call {
                    id: format!("test"),
                    typing: Some(Type::Boolean),
                    location: Location::default(),
                },
                ASTExpression::Call {
                    id: format!("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                ASTExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::IfThenElse {
            condition: Box::new(Expression::Identifier {
                identifier: format!("test"),
            }),
            then_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Identifier {
                        identifier: format!("x"),
                    },
                }],
            },
            else_branch: Block {
                statements: vec![Statement::ExpressionLast {
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                }],
            },
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_structure_into_lir_structure() {
        let expression = ASTExpression::Structure {
            name: format!("Point"),
            fields: vec![
                (
                    format!("x"),
                    ASTExpression::Call {
                        id: format!("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    format!("y"),
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Structure(format!("Point"))),
            location: Location::default(),
        };
        let control = Expression::Structure {
            name: format!("Point"),
            fields: vec![
                (
                    format!("x"),
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                ),
                (
                    format!("y"),
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_array_into_lir_array() {
        let expression = ASTExpression::Array {
            elements: vec![ASTExpression::Call {
                id: format!("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Array(Box::new(Type::Integer), 1)),
            location: Location::default(),
        };
        let control = Expression::Array {
            elements: vec![Expression::Identifier {
                identifier: format!("x"),
            }],
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_match_into_lir_match() {
        let expression = ASTExpression::Match {
            expression: Box::new(ASTExpression::Call {
                id: format!("p"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: format!("Point"),
                        fields: vec![
                            (
                                format!("x"),
                                Pattern::Identifier {
                                    name: format!("x"),
                                    location: Location::default(),
                                },
                            ),
                            (
                                format!("y"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    ASTExpression::Call {
                        id: format!("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    Pattern::Default {
                        location: Location::default(),
                    },
                    None,
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: format!("p"),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: format!("Point"),
                        fields: vec![
                            (
                                format!("x"),
                                Pattern::Identifier {
                                    name: format!("x"),
                                    location: Location::default(),
                                },
                            ),
                            (
                                format!("y"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                ),
                (
                    Pattern::Default {
                        location: Location::default(),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_when_into_lir_match() {
        let expression = ASTExpression::When {
            id: format!("x"),
            option: Box::new(ASTExpression::Call {
                id: format!("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            present: Box::new(ASTExpression::Application {
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
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ],
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            default: Box::new(ASTExpression::Constant {
                constant: Constant::Integer(1),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: format!("x"),
            }),
            arms: vec![
                (
                    Pattern::Some {
                        pattern: Box::new(Pattern::Identifier {
                            name: format!("x"),
                            location: Location::default(),
                        }),
                        location: Location::default(),
                    },
                    None,
                    Expression::FunctionCall {
                        function: Box::new(Expression::Identifier {
                            identifier: format!(" + "),
                        }),
                        arguments: vec![
                            Expression::Identifier {
                                identifier: format!("x"),
                            },
                            Expression::Literal {
                                literal: Constant::Integer(1),
                            },
                        ],
                    },
                ),
                (
                    Pattern::None {
                        location: Location::default(),
                    },
                    None,
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ),
            ],
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_abstraction_into_lir_node_call() {
        let expression = ASTExpression::TypedAbstraction {
            inputs: vec![(format!("x"), Type::Integer)],
            expression: Box::new(ASTExpression::Application {
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
                    ASTExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ],
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
            location: Location::default(),
        };
        let control = Expression::Lambda {
            inputs: vec![(format!("x"), Type::Integer)],
            output: Type::Integer,
            body: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    identifier: format!(" + "),
                }),
                arguments: vec![
                    Expression::Identifier {
                        identifier: format!("x"),
                    },
                    Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                ],
            }),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_field_access_into_lir_field_access() {
        let expression = ASTExpression::FieldAccess {
            expression: Box::new(ASTExpression::Call {
                id: format!("p"),
                typing: Some(Type::Structure("Point".to_string())),
                location: Location::default(),
            }),
            field: format!("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::FieldAccess {
            expression: Box::new(Expression::Identifier {
                identifier: format!("p"),
            }),
            field: format!("x"),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_map_into_lir_map() {
        let expression = ASTExpression::Map {
            expression: Box::new(ASTExpression::Call {
                id: format!("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Box::new(ASTExpression::Call {
                id: format!("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Float))),
                location: Location::default(),
            }),
            typing: Some(Type::Array(Box::new(Type::Float), 3)),
            location: Location::default(),
        };
        let control = Expression::Map {
            mapped: Box::new(Expression::Identifier {
                identifier: format!("a"),
            }),
            function: Box::new(Expression::Identifier {
                identifier: format!("f"),
            }),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_fold_into_lir_fold() {
        let expression = ASTExpression::Fold {
            expression: Box::new(ASTExpression::Call {
                id: format!("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            initialization_expression: Box::new(ASTExpression::Constant {
                constant: Constant::Integer(0),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            function_expression: Box::new(ASTExpression::Call {
                id: format!("sum"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = Expression::Fold {
            folded: Box::new(Expression::Identifier {
                identifier: format!("a"),
            }),
            initialization: Box::new(Expression::Literal {
                literal: Constant::Integer(0),
            }),
            function: Box::new(Expression::Identifier {
                identifier: format!("sum"),
            }),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_sort_into_lir_sort() {
        let expression = ASTExpression::Sort {
            expression: Box::new(ASTExpression::Call {
                id: format!("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Box::new(ASTExpression::Call {
                id: format!("diff"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            }),
            typing: Some(Type::Array(Box::new(Type::Float), 3)),
            location: Location::default(),
        };
        let control = Expression::Sort {
            sorted: Box::new(Expression::Identifier {
                identifier: format!("a"),
            }),
            function: Box::new(Expression::Identifier {
                identifier: format!("diff"),
            }),
        };
        assert_eq!(lir_from_hir(expression), control)
    }

    #[test]
    fn should_transform_ast_zip_into_lir_zip() {
        let expression = ASTExpression::Zip {
            arrays: vec![
                ASTExpression::Call {
                    id: String::from("a"),
                    typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                    location: Location::default(),
                },
                ASTExpression::Call {
                    id: String::from("b"),
                    typing: Some(Type::Array(Box::new(Type::Float), 3)),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Array(
                Box::new(Type::Tuple(vec![Type::Integer, Type::Float])),
                3,
            )),
            location: Location::default(),
        };
        let control = Expression::Zip {
            arrays: vec![
                Expression::Identifier {
                    identifier: "a".to_string(),
                },
                Expression::Identifier {
                    identifier: "b".to_string(),
                },
            ],
        };
        assert_eq!(lir_from_hir(expression), control)
    }
}

#[cfg(test)]
mod get_imports {
    use crate::{
        ast::expression::Expression,
        common::{location::Location, operator::UnaryOperator, r#type::Type},
        lir::item::node_file::import::Import,
    };

    #[test]
    fn should_get_function_import_from_function_call_expression() {
        let expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: format!("my_function"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: format!("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = vec![Import::Function(format!("my_function"))];
        assert_eq!(expression.get_imports(), control)
    }

    #[test]
    fn should_not_import_builtin_functions() {
        let expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: UnaryOperator::Neg.to_string(),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: format!("x"),

                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = vec![];
        assert_eq!(expression.get_imports(), control)
    }

    #[test]
    fn should_not_duplicate_imports() {
        let expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: format!("my_function"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Application {
                function_expression: Box::new(Expression::Call {
                    id: format!("my_function"),
                    typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                    location: Location::default(),
                }),
                inputs: vec![Expression::Call {
                    id: format!("x"),

                    typing: Some(Type::Integer),
                    location: Location::default(),
                }],
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let control = vec![Import::Function(format!("my_function"))];
        assert_eq!(expression.get_imports(), control)
    }
}
