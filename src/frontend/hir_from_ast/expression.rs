use std::collections::BTreeMap;

use crate::ast::expression::{Expression, ExpressionKind};
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::{
    dependencies::Dependencies,
    expression::{Expression as HIRExpression, ExpressionKind as HIRExpressionKind},
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl<E> ExpressionKind<E>
where
    E: HIRFromAST,
{
    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression kind and check identifiers good use
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRExpressionKind<E::HIR>, TerminationError> {
        match self {
            ExpressionKind::Constant { constant } => Ok(HIRExpressionKind::Constant { constant }),
            ExpressionKind::Identifier { id } => {
                let id = symbol_table
                    .get_identifier_id(&id, false, location.clone(), &mut vec![])
                    .or_else(|_| {
                        symbol_table.get_function_id(&id, false, location.clone(), errors)
                    })?;
                Ok(HIRExpressionKind::Identifier { id })
            }
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => Ok(HIRExpressionKind::Application {
                function_expression: Box::new(
                    function_expression.hir_from_ast(symbol_table, errors)?,
                ),
                inputs: inputs
                    .into_iter()
                    .map(|input| input.hir_from_ast(symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            ExpressionKind::Structure { name, fields } => {
                let id = symbol_table.get_struct_id(&name, false, location.clone(), errors)?;
                let mut field_ids = symbol_table
                    .get_struct_fields(&id)
                    .clone()
                    .into_iter()
                    .map(|id| (symbol_table.get_name(&id).clone(), id))
                    .collect::<BTreeMap<_, _>>();

                let fields = fields
                    .into_iter()
                    .map(|(field_name, expression)| {
                        let id = field_ids.remove(&field_name).map_or_else(
                            || {
                                let error = Error::UnknownField {
                                    structure_name: name.clone(),
                                    field_name: field_name.clone(),
                                    location: location.clone(),
                                };
                                errors.push(error);
                                Err(TerminationError)
                            },
                            |id| Ok(id),
                        )?;
                        let expression = expression.hir_from_ast(symbol_table, errors)?;
                        Ok((id, expression))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                // check if there are no missing fields
                field_ids
                    .keys()
                    .map(|field_name| {
                        let error = Error::MissingField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err::<(), TerminationError>(TerminationError)
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(HIRExpressionKind::Structure { id, fields })
            }
            ExpressionKind::Enumeration {
                enum_name,
                elem_name,
            } => {
                let enum_id =
                    symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
                let elem_id = symbol_table.get_identifier_id(
                    &format!("{enum_name}::{elem_name}"),
                    false,
                    location.clone(),
                    errors,
                )?;
                // TODO check elem is in enum
                Ok(HIRExpressionKind::Enumeration { enum_id, elem_id })
            }
            ExpressionKind::Array { elements } => Ok(HIRExpressionKind::Array {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.hir_from_ast(symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            ExpressionKind::Match { expression, arms } => Ok(HIRExpressionKind::Match {
                expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                arms: arms
                    .into_iter()
                    .map(|(pattern, optional_expression, expression)| {
                        symbol_table.local();
                        let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                        let optional_expression = optional_expression
                            .map(|expression| expression.hir_from_ast(symbol_table, errors))
                            .transpose()?;
                        let expression = expression.hir_from_ast(symbol_table, errors)?;
                        symbol_table.global();
                        Ok((pattern, optional_expression, vec![], expression))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            ExpressionKind::When {
                id,
                option,
                present,
                default,
            } => {
                let option = Box::new(option.hir_from_ast(symbol_table, errors)?);
                symbol_table.local();
                let id =
                    symbol_table.insert_identifier(id, None, true, location.clone(), errors)?;
                let present = Box::new(present.hir_from_ast(symbol_table, errors)?);
                let default = Box::new(default.hir_from_ast(symbol_table, errors)?);
                symbol_table.global();
                Ok(HIRExpressionKind::When {
                    id,
                    option,
                    present_body: vec![],
                    present,
                    default_body: vec![],
                    default,
                })
            }
            ExpressionKind::FieldAccess { expression, field } => {
                Ok(HIRExpressionKind::FieldAccess {
                    expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                    field,
                })
            }
            ExpressionKind::TupleElementAccess {
                expression,
                element_number,
            } => Ok(HIRExpressionKind::TupleElementAccess {
                expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                element_number,
            }),
            ExpressionKind::Map {
                expression,
                function_expression,
            } => Ok(HIRExpressionKind::Map {
                expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                function_expression: Box::new(
                    function_expression.hir_from_ast(symbol_table, errors)?,
                ),
            }),
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => Ok(HIRExpressionKind::Fold {
                expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                initialization_expression: Box::new(
                    initialization_expression.hir_from_ast(symbol_table, errors)?,
                ),
                function_expression: Box::new(
                    function_expression.hir_from_ast(symbol_table, errors)?,
                ),
            }),
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => Ok(HIRExpressionKind::Sort {
                expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                function_expression: Box::new(
                    function_expression.hir_from_ast(symbol_table, errors)?,
                ),
            }),
            ExpressionKind::Zip { arrays } => Ok(HIRExpressionKind::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|array| array.hir_from_ast(symbol_table, errors))
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            ExpressionKind::Abstraction { inputs, expression } => todo!(),
            ExpressionKind::TypedAbstraction { inputs, expression } => todo!(),
        }
    }
}

impl HIRFromAST for Expression {
    type HIR = HIRExpression;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Expression { kind, location } = self;
        Ok(HIRExpression {
            kind: kind.hir_from_ast(&location, symbol_table, errors)?,
            typing: None,
            location,
            dependencies: Dependencies::new(),
        })
    }
}
