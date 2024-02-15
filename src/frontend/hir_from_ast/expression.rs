use crate::ast::expression::{Expression, ExpressionKind};
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::pattern::hir_from_ast as pattern_hir_from_ast;
use crate::hir::{
    dependencies::Dependencies,
    expression::{Expression as HIRExpression, ExpressionKind as HIRExpressionKind},
};
use crate::symbol_table::SymbolTable;

/// Transform AST stream expressions into HIR stream expressions.
pub fn hir_from_ast(
    expression: Expression,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRExpression, TerminationError> {
    let Expression {
        kind,
        location,
    } = expression;

    match kind {
        ExpressionKind::Constant { constant } => Ok(HIRExpression {
            kind: HIRExpressionKind::Constant { constant },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Identifier { id } => {
            let id = symbol_table.get_identifier_id(&id, false, location, errors)?;
            Ok(HIRExpression {
                kind: HIRExpressionKind::Identifier { id },
                typing: None,
                location,
                dependencies: Dependencies::new(),
            })
        }
        ExpressionKind::Application {
            function_expression,
            inputs,
        } => Ok(HIRExpression {
            kind: HIRExpressionKind::Application {
                function_expression: Box::new(hir_from_ast(
                    *function_expression,
                    symbol_table,
                    errors,
                )?),
                inputs: inputs
                    .into_iter()
                    .map(|input| hir_from_ast(input, symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Structure { name, fields } => {
            let id = symbol_table.get_identifier_id(&name, false, location, errors)?;
            // TODO check fields are all in structure
            Ok(HIRExpression {
                kind: HIRExpressionKind::Structure {
                    id,
                    fields: fields
                        .into_iter()
                        .map(|(field, expression)| {
                            let id = symbol_table.get_identifier_id(
                                &field,
                                false,
                                location.clone(),
                                errors,
                            )?;
                            let expression = hir_from_ast(expression, symbol_table, errors)?;
                            Ok((id, expression))
                        })
                        .collect::<Vec<Result<_, _>>>()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()?,
                },
                typing: None,
                location,
                dependencies: Dependencies::new(),
            })
        }
        ExpressionKind::Enumeration {
            enum_name,
            elem_name,
        } => {
            let enum_id =
                symbol_table.get_identifier_id(&enum_name, false, location.clone(), errors)?;
            let elem_id =
                symbol_table.get_identifier_id(&elem_name, false, location.clone(), errors)?;
            // TODO check elem is in enum
            Ok(HIRExpression {
                kind: HIRExpressionKind::Enumeration { enum_id, elem_id },
                typing: None,
                location,
                dependencies: Dependencies::new(),
            })
        }
        ExpressionKind::Array { elements } => Ok(HIRExpression {
            kind: HIRExpressionKind::Array {
                elements: elements
                    .into_iter()
                    .map(|expression| hir_from_ast(expression, symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Match { expression, arms } => Ok(HIRExpression {
            kind: HIRExpressionKind::Match {
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
                arms: arms
                    .into_iter()
                    .map(|(pattern, optional_expression, expression)| {
                        symbol_table.local();
                        let pattern = pattern_hir_from_ast(pattern, symbol_table, errors)?;
                        let optional_expression = optional_expression
                            .map(|expression| hir_from_ast(expression, symbol_table, errors))
                            .transpose()?;
                        let expression = hir_from_ast(expression, symbol_table, errors)?;
                        symbol_table.global();
                        Ok((pattern, optional_expression, vec![], expression))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::When {
            id,
            option,
            present,
            default,
        } => {
            symbol_table.local();
            let id = symbol_table.insert_identifier(id, None, true, location.clone(), errors)?;
            let option = Box::new(hir_from_ast(*option, symbol_table, errors)?);
            let present = Box::new(hir_from_ast(*present, symbol_table, errors)?);
            let default = Box::new(hir_from_ast(*default, symbol_table, errors)?);
            symbol_table.global();
            Ok(HIRExpression {
                kind: HIRExpressionKind::When {
                    id,
                    option,
                    present_body: vec![],
                    present,
                    default_body: vec![],
                    default,
                },
                typing: None,
                location,
                dependencies: Dependencies::new(),
            })
        }
        ExpressionKind::FieldAccess { expression, field } => Ok(HIRExpression {
            kind: HIRExpressionKind::FieldAccess {
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
                field,
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::TupleElementAccess {
            expression,
            element_number,
        } => Ok(HIRExpression {
            kind: HIRExpressionKind::TupleElementAccess {
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
                element_number,
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Map {
            expression,
            function_expression,
        } => Ok(HIRExpression {
            kind: HIRExpressionKind::Map {
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
                function_expression: Box::new(hir_from_ast(
                    *function_expression,
                    symbol_table,
                    errors,
                )?),
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Fold {
            expression,
            initialization_expression,
            function_expression,
        } => Ok(HIRExpression {
            kind: HIRExpressionKind::Fold {
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
                initialization_expression: Box::new(hir_from_ast(
                    *initialization_expression,
                    symbol_table,
                    errors,
                )?),
                function_expression: Box::new(hir_from_ast(
                    *function_expression,
                    symbol_table,
                    errors,
                )?),
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Sort {
            expression,
            function_expression,
        } => Ok(HIRExpression {
            kind: HIRExpressionKind::Sort {
                expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
                function_expression: Box::new(hir_from_ast(
                    *function_expression,
                    symbol_table,
                    errors,
                )?),
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Zip { arrays } => Ok(HIRExpression {
            kind: HIRExpressionKind::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|array| hir_from_ast(array, symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            },
            typing: None,
            location,
            dependencies: Dependencies::new(),
        }),
        ExpressionKind::Abstraction { inputs, expression } => todo!(),
        ExpressionKind::TypedAbstraction { inputs, expression } => todo!(),
    }
}
