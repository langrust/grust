use std::collections::HashMap;

use crate::ast::expression::{
    Application, Arm, Array, Enumeration, Expression, FieldAccess, Fold, Map, Match, Sort,
    Structure, Tuple, TupleElementAccess, TypedAbstraction, Zip,
};
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::{
    dependencies::Dependencies,
    expression::{Expression as HIRExpression, ExpressionKind},
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl<E> Application<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Application {
            function_expression,
            inputs,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Application {
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
            inputs: inputs
                .into_iter()
                .map(|input| input.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<E> Structure<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Structure { name, fields } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        let id = symbol_table.get_struct_id(&name, false, location.clone(), errors)?;
        let mut field_ids = symbol_table
            .get_struct_fields(id)
            .clone()
            .into_iter()
            .map(|id| (symbol_table.get_name(id).clone(), id))
            .collect::<HashMap<_, _>>();

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

        Ok(ExpressionKind::Structure { id, fields })
    }
}

impl Enumeration {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast<E>(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError>
    where
        E: HIRFromAST,
    {
        let Enumeration {
            enum_name,
            elem_name,
        } = self;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        let enum_id = symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
        let elem_id = symbol_table.get_enum_elem_id(
            &elem_name,
            &enum_name,
            false,
            location.clone(),
            errors,
        )?;
        // TODO check elem is in enum
        Ok(ExpressionKind::Enumeration { enum_id, elem_id })
    }
}

impl<E> Array<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Array { elements } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Array {
            elements: elements
                .into_iter()
                .map(|expression| expression.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<E> Tuple<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Tuple { elements } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Tuple {
            elements: elements
                .into_iter()
                .map(|expression| expression.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<E> Match<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Match { expression, arms } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Match {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            arms: arms
                .into_iter()
                .map(
                    |Arm {
                         pattern,
                         guard,
                         expression,
                     }| {
                        symbol_table.local();
                        let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                        let guard = guard
                            .map(|expression| expression.hir_from_ast(symbol_table, errors))
                            .transpose()?;
                        let expression = expression.hir_from_ast(symbol_table, errors)?;
                        symbol_table.global();
                        Ok((pattern, guard, vec![], expression))
                    },
                )
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<E> FieldAccess<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let FieldAccess { expression, field } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::FieldAccess {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            field,
        })
    }
}

impl<E> TupleElementAccess<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let TupleElementAccess {
            expression,
            element_number,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::TupleElementAccess {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            element_number,
        })
    }
}

impl<E> Map<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Map {
            expression,
            function_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Map {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> Fold<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Fold {
            expression,
            initialization_expression,
            function_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Fold {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            initialization_expression: Box::new(
                initialization_expression.hir_from_ast(symbol_table, errors)?,
            ),
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> Sort<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Sort {
            expression,
            function_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Sort {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> Zip<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let Zip { arrays } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(ExpressionKind::Zip {
            arrays: arrays
                .into_iter()
                .map(|array| array.hir_from_ast(symbol_table, errors))
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<E> TypedAbstraction<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<ExpressionKind<E::HIR>, TerminationError> {
        let TypedAbstraction { inputs, expression } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use

        symbol_table.local();
        let inputs = inputs
            .into_iter()
            .map(|(input_name, typing)| {
                let typing = typing.hir_from_ast(&location, symbol_table, errors)?;
                symbol_table.insert_identifier(
                    input_name,
                    Some(typing),
                    true,
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let expression = expression.hir_from_ast(symbol_table, errors)?;
        symbol_table.global();

        Ok(ExpressionKind::Abstraction {
            inputs,
            expression: Box::new(expression),
        })
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
        let location = Location::default();
        let kind = match self {
            Expression::Constant(constant) => ExpressionKind::Constant { constant },
            Expression::Identifier(id) => {
                let id = symbol_table
                    .get_identifier_id(&id, false, location.clone(), &mut vec![])
                    .or_else(|_| {
                        symbol_table.get_function_id(&id, false, location.clone(), errors)
                    })?;
                ExpressionKind::Identifier { id }
            }
            Expression::Application(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::TypedAbstraction(expression) => {
                expression.hir_from_ast(&location, symbol_table, errors)?
            }
            Expression::Structure(expression) => {
                expression.hir_from_ast(&location, symbol_table, errors)?
            }
            Expression::Tuple(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::Enumeration(expression) => {
                expression.hir_from_ast::<Expression>(&location, symbol_table, errors)?
            }
            Expression::Array(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::Match(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::FieldAccess(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::TupleElementAccess(expression) => {
                expression.hir_from_ast(symbol_table, errors)?
            }
            Expression::Map(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::Fold(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::Sort(expression) => expression.hir_from_ast(symbol_table, errors)?,
            Expression::Zip(expression) => expression.hir_from_ast(symbol_table, errors)?,
        };
        Ok(HIRExpression {
            kind,
            typing: None,
            location,
            dependencies: Dependencies::new(),
        })
    }
}
