use crate::{
    common::{operator::OtherOperator, scope::Scope},
    frontend::lir_from_hir::{
        pattern::lir_from_hir as pattern_lir_from_hir,
        statement::lir_from_hir as statement_lir_from_hir,
    },
    hir::expression::{Expression, ExpressionKind},
    lir::{
        block::Block,
        expression::{Expression as LIRExpression, FieldIdentifier},
        pattern::Pattern,
        statement::Statement,
    },
    symbol_table::SymbolTable,
};

/// Transform HIR expression into LIR expression.
pub fn lir_from_hir(expression: Expression, symbol_table: &SymbolTable) -> LIRExpression {
    match expression.kind {
        ExpressionKind::Constant { constant, .. } => LIRExpression::Literal { literal: constant },
        ExpressionKind::Identifier { id, .. } => {
            let scope = symbol_table.get_scope(&id);
            let name = symbol_table.get_name(&id).clone();
            match scope {
                Scope::Input => LIRExpression::InputAccess { identifier: name },
                Scope::Memory => LIRExpression::MemoryAccess { identifier: name },
                Scope::Output | Scope::Local => LIRExpression::Identifier { identifier: name },
            }
        }
        ExpressionKind::Application {
            function_expression,
            mut inputs,
            ..
        } => match function_expression.kind {
            ExpressionKind::Identifier { id, .. }
                if OtherOperator::IfThenElse
                    .to_string()
                    .eq(symbol_table.get_name(&id)) =>
            {
                assert!(inputs.len() == 3);
                let else_branch = lir_from_hir(inputs.pop().unwrap(), symbol_table);
                let then_branch = lir_from_hir(inputs.pop().unwrap(), symbol_table);
                let condition = lir_from_hir(inputs.pop().unwrap(), symbol_table);
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
                let arguments = inputs
                    .into_iter()
                    .map(|input| lir_from_hir(input, symbol_table))
                    .collect();
                LIRExpression::FunctionCall {
                    function: Box::new(lir_from_hir(*function_expression, symbol_table)),
                    arguments,
                }
            }
        },
        ExpressionKind::Abstraction {
            inputs, expression, ..
        } => {
            let inputs = inputs
                .iter()
                .map(|id| {
                    (
                        symbol_table.get_name(id).clone(),
                        symbol_table.get_type(id).clone(),
                    )
                })
                .collect();
            let output = expression.get_type().expect("it should be typed").clone();
            LIRExpression::Lambda {
                inputs,
                output,
                body: Box::new(lir_from_hir(*expression, symbol_table)),
            }
        }
        ExpressionKind::Structure { id, fields, .. } => LIRExpression::Structure {
            name: symbol_table.get_name(&id).clone(),
            fields: fields
                .into_iter()
                .map(|(id, expression)| {
                    (
                        symbol_table.get_name(&id).clone(),
                        lir_from_hir(expression, symbol_table),
                    )
                })
                .collect(),
        },
        ExpressionKind::Enumeration { enum_id, elem_id } => LIRExpression::Enumeration {
            name: symbol_table.get_name(&enum_id).clone(),
            element: symbol_table.get_name(&elem_id).clone(),
        },
        ExpressionKind::Array { elements, .. } => LIRExpression::Array {
            elements: elements
                .into_iter()
                .map(|element| lir_from_hir(element, symbol_table))
                .collect(),
        },
        ExpressionKind::Match {
            expression, arms, ..
        } => LIRExpression::Match {
            matched: Box::new(lir_from_hir(*expression, symbol_table)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, body, expression)| {
                    (
                        pattern_lir_from_hir(pattern, symbol_table),
                        guard.map(|expression| lir_from_hir(expression, symbol_table)),
                        if body.is_empty() {
                            lir_from_hir(expression, symbol_table)
                        } else {
                            let mut statements = body
                                .into_iter()
                                .map(|statement| statement_lir_from_hir(statement, symbol_table))
                                .collect::<Vec<_>>();
                            statements.push(Statement::ExpressionLast {
                                expression: lir_from_hir(expression, symbol_table),
                            });
                            LIRExpression::Block {
                                block: Block { statements },
                            }
                        },
                    )
                })
                .collect(),
        },
        ExpressionKind::When {
            id,
            option,
            present,
            default,
            ..
        } => {
            let typing = symbol_table.get_type(&id).clone();
            LIRExpression::Match {
                matched: Box::new(lir_from_hir(*option, symbol_table)),
                arms: vec![
                    (
                        Pattern::Some {
                            pattern: Box::new(Pattern::Identifier {
                                name: symbol_table.get_name(&id).clone(),
                            }),
                        },
                        None,
                        lir_from_hir(*present, symbol_table),
                    ),
                    (Pattern::None, None, lir_from_hir(*default, symbol_table)),
                ],
            }
        }
        ExpressionKind::FieldAccess {
            expression, field, ..
        } => LIRExpression::FieldAccess {
            expression: Box::new(lir_from_hir(*expression, symbol_table)),
            field: FieldIdentifier::Named(field),
        },
        ExpressionKind::TupleElementAccess {
            expression,
            element_number,
            ..
        } => LIRExpression::FieldAccess {
            expression: Box::new(lir_from_hir(*expression, symbol_table)),
            field: FieldIdentifier::Unamed(element_number),
        },
        ExpressionKind::Abstraction { .. } => {
            unreachable!()
        }
        ExpressionKind::Map {
            expression,
            function_expression,
            ..
        } => LIRExpression::Map {
            mapped: Box::new(lir_from_hir(*expression, symbol_table)),
            function: Box::new(lir_from_hir(*function_expression, symbol_table)),
        },
        ExpressionKind::Fold {
            expression,
            initialization_expression,
            function_expression,
            ..
        } => LIRExpression::Fold {
            folded: Box::new(lir_from_hir(*expression, symbol_table)),
            initialization: Box::new(lir_from_hir(*initialization_expression, symbol_table)),
            function: Box::new(lir_from_hir(*function_expression, symbol_table)),
        },
        ExpressionKind::Sort {
            expression,
            function_expression,
            ..
        } => LIRExpression::Sort {
            sorted: Box::new(lir_from_hir(*expression, symbol_table)),
            function: Box::new(lir_from_hir(*function_expression, symbol_table)),
        },
        ExpressionKind::Zip { arrays, .. } => LIRExpression::Zip {
            arrays: arrays
                .into_iter()
                .map(|element| lir_from_hir(element, symbol_table))
                .collect(),
        },
    }
}
