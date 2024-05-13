use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    common::{
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        r#type::Type,
        scope::Scope,
    },
    hir::expression::{Expression, ExpressionKind},
    lir::{
        block::Block,
        expression::{Expression as LIRExpression, FieldIdentifier},
        item::import::Import,
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
                let name = symbol_table.get_name(id).clone();
                if symbol_table.is_function(id) {
                    LIRExpression::Identifier { identifier: name }
                } else {
                    let scope = symbol_table.get_scope(id);
                    match scope {
                        Scope::Input => LIRExpression::InputAccess { identifier: name },
                        Scope::Memory => LIRExpression::MemoryAccess { identifier: name },
                        Scope::Output | Scope::Local => {
                            LIRExpression::Identifier { identifier: name }
                        }
                    }
                }
            }
            ExpressionKind::Unop { op, expression } => {
                let expression = expression.lir_from_hir(symbol_table);
                LIRExpression::Unop {
                    op,
                    expression: Box::new(expression),
                }
            }
            ExpressionKind::Binop {
                op,
                left_expression,
                right_expression,
            } => {
                let left_expression = left_expression.lir_from_hir(symbol_table);
                let right_expression = right_expression.lir_from_hir(symbol_table);
                LIRExpression::Binop {
                    op,
                    left_expression: Box::new(left_expression),
                    right_expression: Box::new(right_expression),
                }
            }
            ExpressionKind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                let else_branch = expression.lir_from_hir(symbol_table);
                let then_branch = true_expression.lir_from_hir(symbol_table);
                let condition = false_expression.lir_from_hir(symbol_table);
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
            ExpressionKind::Application {
                function_expression,
                inputs,
                ..
            } => {
                let arguments = inputs
                    .into_iter()
                    .map(|input| input.lir_from_hir(symbol_table))
                    .collect();
                LIRExpression::FunctionCall {
                    function: Box::new(function_expression.lir_from_hir(symbol_table)),
                    arguments,
                }
            }
            ExpressionKind::Abstraction {
                inputs, expression, ..
            } => {
                let inputs = inputs
                    .iter()
                    .map(|id| {
                        (
                            symbol_table.get_name(*id).clone(),
                            symbol_table.get_type(*id).clone(),
                        )
                    })
                    .collect();
                let output = expression.get_type().expect("it should be typed").clone();
                LIRExpression::Lambda {
                    inputs,
                    output,
                    body: Box::new(LIRExpression::Block {
                        block: Block {
                            statements: vec![Statement::ExpressionLast {
                                expression: expression.lir_from_hir(symbol_table),
                            }],
                        },
                    }),
                }
            }
            ExpressionKind::Structure { id, fields, .. } => LIRExpression::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expression.lir_from_hir(symbol_table),
                        )
                    })
                    .collect(),
            },
            ExpressionKind::Enumeration { enum_id, elem_id } => LIRExpression::Enumeration {
                name: symbol_table.get_name(enum_id).clone(),
                element: symbol_table.get_name(elem_id).clone(),
            },
            ExpressionKind::Array { elements } => LIRExpression::Array {
                elements: elements
                    .into_iter()
                    .map(|element| element.lir_from_hir(symbol_table))
                    .collect(),
            },
            ExpressionKind::Tuple { elements } => LIRExpression::Tuple {
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
            } => LIRExpression::Match {
                matched: Box::new(option.lir_from_hir(symbol_table)),
                arms: vec![
                    (
                        Pattern::Some {
                            pattern: Box::new(Pattern::Identifier {
                                name: symbol_table.get_name(id).clone(),
                            }),
                        },
                        None,
                        present.lir_from_hir(symbol_table),
                    ),
                    (Pattern::None, None, default.lir_from_hir(symbol_table)),
                ],
            },
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
                .eq(symbol_table.get_name(*id)),
            _ => false,
        }
    }

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match self {
            ExpressionKind::Constant { .. } => vec![],
            ExpressionKind::Identifier { id } => {
                if symbol_table.is_function(*id) {
                    if let Some(_) = BinaryOperator::iter()
                        .find(|binary| binary.to_string().eq(symbol_table.get_name(*id)))
                    {
                        vec![]
                    } else if let Some(_) = UnaryOperator::iter()
                        .find(|unary| unary.to_string().eq(symbol_table.get_name(*id)))
                    {
                        vec![]
                    } else if let Some(_) = OtherOperator::iter()
                        .find(|op| op.to_string().eq(symbol_table.get_name(*id)))
                    {
                        vec![]
                    } else {
                        vec![Import::Function(symbol_table.get_name(*id).clone())]
                    }
                } else {
                    vec![]
                }
            }
            ExpressionKind::Unop { expression, .. } => expression.get_imports(symbol_table),
            ExpressionKind::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                let mut imports = left_expression.get_imports(symbol_table);
                let mut right_imports = right_expression.get_imports(symbol_table);
                imports.append(&mut right_imports);
                imports
            }
            ExpressionKind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                let mut imports = expression.get_imports(symbol_table);
                let mut true_imports = true_expression.get_imports(symbol_table);
                let mut false_imports = false_expression.get_imports(symbol_table);
                imports.append(&mut true_imports);
                imports.append(&mut false_imports);
                imports
            }
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                let mut imports = function_expression.get_imports(symbol_table);
                let mut inputs_imports = inputs
                    .iter()
                    .flat_map(|expression| expression.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.append(&mut inputs_imports);
                imports
            }
            ExpressionKind::Abstraction { expression, .. } => expression.get_imports(symbol_table),
            ExpressionKind::Structure { id, fields } => {
                let mut imports = fields
                    .iter()
                    .flat_map(|(_, expression)| expression.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.push(Import::Structure(symbol_table.get_name(*id).clone()));

                imports
            }
            ExpressionKind::Enumeration { enum_id, .. } => {
                vec![Import::Enumeration(symbol_table.get_name(*enum_id).clone())]
            }
            ExpressionKind::Array { elements } | ExpressionKind::Tuple { elements } => elements
                .iter()
                .flat_map(|expression| expression.get_imports(symbol_table))
                .unique()
                .collect(),
            ExpressionKind::Match { expression, arms } => {
                let mut imports = expression.get_imports(symbol_table);
                let mut arms_imports = arms
                    .iter()
                    .flat_map(|(pattern, option, statements, expression)| {
                        let mut pattern_imports = pattern.get_imports(symbol_table);
                        let mut option_imports = option
                            .as_ref()
                            .map_or(vec![], |expression| expression.get_imports(symbol_table));
                        pattern_imports.append(&mut option_imports);
                        let mut statements_imports = statements
                            .iter()
                            .flat_map(|statement| statement.get_imports(symbol_table))
                            .unique()
                            .collect::<Vec<_>>();
                        pattern_imports.append(&mut statements_imports);
                        let mut expression_imports = expression.get_imports(symbol_table);
                        pattern_imports.append(&mut expression_imports);
                        pattern_imports
                    })
                    .unique()
                    .collect::<Vec<_>>();
                imports.append(&mut arms_imports);
                imports
            }
            ExpressionKind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                let mut imports = option.get_imports(symbol_table);
                let mut present_imports = present.get_imports(symbol_table);
                imports.append(&mut present_imports);
                let mut present_body_imports = present_body
                    .iter()
                    .flat_map(|statement| statement.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.append(&mut present_body_imports);
                let mut default_imports = default.get_imports(symbol_table);
                imports.append(&mut default_imports);
                let mut default_body_imports = default_body
                    .iter()
                    .flat_map(|statement| statement.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.append(&mut default_body_imports);
                imports
            }
            ExpressionKind::FieldAccess { expression, .. } => expression.get_imports(symbol_table),
            ExpressionKind::TupleElementAccess { expression, .. } => {
                expression.get_imports(symbol_table)
            }
            ExpressionKind::Map {
                expression,
                function_expression,
            } => {
                let mut imports = expression.get_imports(symbol_table);
                let mut function_imports = function_expression.get_imports(symbol_table);
                imports.append(&mut function_imports);
                imports
            }
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                let mut imports = expression.get_imports(symbol_table);
                let mut initialization_imports =
                    initialization_expression.get_imports(symbol_table);
                imports.append(&mut initialization_imports);
                let mut function_imports = function_expression.get_imports(symbol_table);
                imports.append(&mut function_imports);
                imports
            }
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => {
                let mut imports = expression.get_imports(symbol_table);
                let mut function_imports = function_expression.get_imports(symbol_table);
                imports.append(&mut function_imports);
                imports
            }
            ExpressionKind::Zip { arrays } => arrays
                .iter()
                .flat_map(|expression| expression.get_imports(symbol_table))
                .unique()
                .collect(),
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

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        self.kind.get_imports(symbol_table)
    }
}
