use crate::{
    common::{operator::OtherOperator, r#type::Type, scope::Scope},
    hir::expression::{Expression, ExpressionKind},
    lir::{
        block::Block,
        expression::{Expression as LIRExpression, FieldIdentifier},
        pattern::Pattern,
        statement::Statement,
    },
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl<E> LIRFromHIR for ExpressionKind<E>
where
    E: LIRFromHIR<LIR = LIRExpression>,
{
    type LIR = LIRExpression;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self {
            ExpressionKind::Constant { constant, .. } => {
                LIRExpression::Literal { literal: constant }
            }
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
            } => {
                if function_expression.is_if_then_else(symbol_table) {
                    assert!(inputs.len() == 3);
                    let else_branch = inputs.pop().unwrap().lir_from_hir(symbol_table);
                    let then_branch = inputs.pop().unwrap().lir_from_hir(symbol_table);
                    let condition = inputs.pop().unwrap().lir_from_hir(symbol_table);
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
                } else {
                    let arguments = inputs
                        .into_iter()
                        .map(|input| input.lir_from_hir(symbol_table))
                        .collect();
                    LIRExpression::FunctionCall {
                        function: Box::new(function_expression.lir_from_hir(symbol_table)),
                        arguments,
                    }
                }
            }
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
                    body: Box::new(expression.lir_from_hir(symbol_table)),
                }
            }
            ExpressionKind::Structure { id, fields, .. } => LIRExpression::Structure {
                name: symbol_table.get_name(&id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(&id).clone(),
                            expression.lir_from_hir(symbol_table),
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
                    .map(|element| element.lir_from_hir(symbol_table))
                    .collect(),
            },
            ExpressionKind::Match {
                expression, arms, ..
            } => LIRExpression::Match {
                matched: Box::new(expression.lir_from_hir(symbol_table)),
                arms: arms
                    .into_iter()
                    .map(|(pattern, guard, body, expression)| {
                        (
                            pattern.lir_from_hir(symbol_table),
                            guard.map(|expression| expression.lir_from_hir(symbol_table)),
                            if body.is_empty() {
                                expression.lir_from_hir(symbol_table)
                            } else {
                                let mut statements = body
                                    .into_iter()
                                    .map(|statement| statement.lir_from_hir(symbol_table))
                                    .collect::<Vec<_>>();
                                statements.push(Statement::ExpressionLast {
                                    expression: expression.lir_from_hir(symbol_table),
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
                    matched: Box::new(option.lir_from_hir(symbol_table)),
                    arms: vec![
                        (
                            Pattern::Some {
                                pattern: Box::new(Pattern::Identifier {
                                    name: symbol_table.get_name(&id).clone(),
                                }),
                            },
                            None,
                            present.lir_from_hir(symbol_table),
                        ),
                        (Pattern::None, None, default.lir_from_hir(symbol_table)),
                    ],
                }
            }
            ExpressionKind::FieldAccess {
                expression, field, ..
            } => LIRExpression::FieldAccess {
                expression: Box::new(expression.lir_from_hir(symbol_table)),
                field: FieldIdentifier::Named(field),
            },
            ExpressionKind::TupleElementAccess {
                expression,
                element_number,
                ..
            } => LIRExpression::FieldAccess {
                expression: Box::new(expression.lir_from_hir(symbol_table)),
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
                mapped: Box::new(expression.lir_from_hir(symbol_table)),
                function: Box::new(function_expression.lir_from_hir(symbol_table)),
            },
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
                ..
            } => LIRExpression::Fold {
                folded: Box::new(expression.lir_from_hir(symbol_table)),
                initialization: Box::new(initialization_expression.lir_from_hir(symbol_table)),
                function: Box::new(function_expression.lir_from_hir(symbol_table)),
            },
            ExpressionKind::Sort {
                expression,
                function_expression,
                ..
            } => LIRExpression::Sort {
                sorted: Box::new(expression.lir_from_hir(symbol_table)),
                function: Box::new(function_expression.lir_from_hir(symbol_table)),
            },
            ExpressionKind::Zip { arrays, .. } => LIRExpression::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|element| element.lir_from_hir(symbol_table))
                    .collect(),
            },
        }
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            ExpressionKind::Identifier { id, .. } => OtherOperator::IfThenElse
                .to_string()
                .eq(symbol_table.get_name(&id)),
            _ => false,
        }
    }
}

impl LIRFromHIR for Expression {
    type LIR = LIRExpression;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        self.kind.lir_from_hir(symbol_table)
    }
    fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        self.kind.is_if_then_else(symbol_table)
    }
}
