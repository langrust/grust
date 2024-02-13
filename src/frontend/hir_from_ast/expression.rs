use crate::ast::expression::Expression;
use crate::common::scope::Scope;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::pattern::hir_from_ast as pattern_hir_from_ast;
use crate::hir::expression::Expression as HIRExpression;
use crate::symbol_table::SymbolTable;

/// Transform AST stream expressions into HIR stream expressions.
pub fn hir_from_ast(
    expression: Expression,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRExpression, TerminationError> {
    match expression {
        Expression::Constant {
            constant,
            typing,
            location,
        } => Ok(HIRExpression::Constant {
            constant,
            typing: None,
            location,
        }),
        Expression::Call {
            id,
            typing,
            location,
        } => {
            let id = symbol_table.get_identifier_id(&id, false, location, errors)?;
            Ok(HIRExpression::Call {
                id,
                typing: None,
                location,
            })
        }
        Expression::Application {
            function_expression,
            inputs,
            typing,
            location,
        } => Ok(HIRExpression::Application {
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
            typing: None,
            location,
        }),
        Expression::Structure {
            name,
            fields,
            typing,
            location,
        } => Ok(HIRExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(field, expression)| {
                    Ok((field, hir_from_ast(expression, symbol_table, errors)?))
                })
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            typing: None,
            location,
        }),
        Expression::Array {
            elements,
            typing,
            location,
        } => Ok(HIRExpression::Array {
            elements: elements
                .into_iter()
                .map(|expression| hir_from_ast(expression, symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            typing: None,
            location,
        }),
        Expression::Match {
            expression,
            arms,
            typing,
            location,
        } => Ok(HIRExpression::Match {
            expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            arms: arms
                .into_iter()
                .map(|(pattern, optional_expression, expression)| {
                    symbol_table.local();
                    let pattern = pattern_hir_from_ast(pattern, false, symbol_table, errors)?;
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
            typing: None,
            location,
        }),
        Expression::When {
            id,
            option,
            present,
            default,
            typing,
            location,
        } => {
            symbol_table.local();
            let id =
                symbol_table.insert_signal(id, Scope::Local, true, location.clone(), errors)?;
            let option = Box::new(hir_from_ast(*option, symbol_table, errors)?);
            let present = Box::new(hir_from_ast(*present, symbol_table, errors)?);
            let default = Box::new(hir_from_ast(*default, symbol_table, errors)?);
            symbol_table.global();
            Ok(HIRExpression::When {
                id,
                option,
                present_body: vec![],
                present,
                default_body: vec![],
                default,
                typing: None,
                location,
            })
        }
        Expression::FieldAccess {
            expression,
            field,
            typing,
            location,
        } => Ok(HIRExpression::FieldAccess {
            expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            field,
            typing: None,
            location,
        }),
        Expression::TupleElementAccess {
            expression,
            element_number,
            typing,
            location,
        } => Ok(HIRExpression::TupleElementAccess {
            expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            element_number,
            typing: None,
            location,
        }),
        Expression::Map {
            expression,
            function_expression,
            typing,
            location,
        } => Ok(HIRExpression::Map {
            expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            function_expression: Box::new(hir_from_ast(*function_expression, symbol_table, errors)?),
            typing: None,
            location,
        }),
        Expression::Fold {
            expression,
            initialization_expression,
            function_expression,
            typing,
            location,
        } => Ok(HIRExpression::Fold {
            expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            initialization_expression: Box::new(hir_from_ast(
                *initialization_expression,
                symbol_table,
                errors,
            )?),
            function_expression: Box::new(hir_from_ast(*function_expression, symbol_table, errors)?),
            typing: None,
            location,
        }),
        Expression::Sort {
            expression,
            function_expression,
            typing,
            location,
        } => Ok(HIRExpression::Sort {
            expression: Box::new(hir_from_ast(*expression, symbol_table, errors)?),
            function_expression: Box::new(hir_from_ast(*function_expression, symbol_table, errors)?),
            typing: None,
            location,
        }),
        Expression::Zip {
            arrays,
            typing,
            location,
        } => Ok(HIRExpression::Zip {
            arrays: arrays
                .into_iter()
                .map(|array| hir_from_ast(array, symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            typing: None,
            location,
        }),
        Expression::Abstraction {
            inputs,
            expression,
            typing,
            location,
        } => todo!(),
        Expression::TypedAbstraction {
            inputs,
            expression,
            typing,
            location,
        } => todo!(),
    }
}
