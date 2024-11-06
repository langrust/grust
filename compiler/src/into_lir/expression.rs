prelude! {
    lir::{ Block, FieldIdentifier, Stmt },
}

impl<'a, E> IntoLir<&'a SymbolTable> for hir::expr::Kind<E>
where
    E: IntoLir<&'a SymbolTable, Lir = lir::Expr>,
{
    type Lir = lir::Expr;

    fn into_lir(self, symbol_table: &'a SymbolTable) -> Self::Lir {
        match self {
            Self::Constant { constant, .. } => lir::Expr::Literal { literal: constant },
            Self::Identifier { id, .. } => {
                let name = symbol_table.get_name(id).clone();
                if symbol_table.is_function(id) {
                    lir::Expr::Identifier { identifier: name }
                } else {
                    let scope = symbol_table.get_scope(id);
                    match scope {
                        Scope::Input => lir::Expr::InputAccess { identifier: name },
                        Scope::Output | Scope::Local | Scope::VeryLocal => {
                            lir::Expr::Identifier { identifier: name }
                        }
                    }
                }
            }
            Self::Unop { op, expression } => {
                let expression = expression.into_lir(symbol_table);
                lir::Expr::Unop {
                    op,
                    expression: Box::new(expression),
                }
            }
            Self::Binop {
                op,
                left_expression,
                right_expression,
            } => {
                let left_expression = left_expression.into_lir(symbol_table);
                let right_expression = right_expression.into_lir(symbol_table);
                lir::Expr::Binop {
                    op,
                    left_expression: Box::new(left_expression),
                    right_expression: Box::new(right_expression),
                }
            }
            Self::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                let condition = expression.into_lir(symbol_table);
                let then_branch = true_expression.into_lir(symbol_table);
                let else_branch = false_expression.into_lir(symbol_table);
                lir::Expr::IfThenElse {
                    condition: Box::new(condition),
                    then_branch: Block {
                        statements: vec![Stmt::ExprLast {
                            expression: then_branch,
                        }],
                    },
                    else_branch: Block {
                        statements: vec![Stmt::ExprLast {
                            expression: else_branch,
                        }],
                    },
                }
            }
            Self::Application {
                function_expression,
                inputs,
                ..
            } => {
                let arguments = inputs
                    .into_iter()
                    .map(|input| input.into_lir(symbol_table))
                    .collect();
                lir::Expr::FunctionCall {
                    function: Box::new(function_expression.into_lir(symbol_table)),
                    arguments,
                }
            }
            Self::Abstraction {
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
                lir::Expr::Lambda {
                    inputs,
                    output,
                    body: Box::new(lir::Expr::Block {
                        block: Block {
                            statements: vec![Stmt::ExprLast {
                                expression: expression.into_lir(symbol_table),
                            }],
                        },
                    }),
                }
            }
            Self::Structure { id, fields, .. } => lir::Expr::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expression.into_lir(symbol_table),
                        )
                    })
                    .collect(),
            },
            Self::Enumeration { enum_id, elem_id } => lir::Expr::Enumeration {
                name: symbol_table.get_name(enum_id).clone(),
                element: symbol_table.get_name(elem_id).clone(),
            },
            Self::Array { elements } => lir::Expr::Array {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
            Self::Tuple { elements } => lir::Expr::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
            Self::Match {
                expression, arms, ..
            } => lir::Expr::Match {
                matched: Box::new(expression.into_lir(symbol_table)),
                arms: arms
                    .into_iter()
                    .map(|(pattern, guard, body, expression)| {
                        (
                            pattern.into_lir(symbol_table),
                            guard.map(|expression| expression.into_lir(symbol_table)),
                            if body.is_empty() {
                                expression.into_lir(symbol_table)
                            } else {
                                let mut statements = body
                                    .into_iter()
                                    .map(|statement| statement.into_lir(symbol_table))
                                    .collect::<Vec<_>>();
                                statements.push(Stmt::ExprLast {
                                    expression: expression.into_lir(symbol_table),
                                });
                                lir::Expr::Block {
                                    block: Block { statements },
                                }
                            },
                        )
                    })
                    .collect(),
            },
            Self::FieldAccess {
                expression, field, ..
            } => lir::Expr::FieldAccess {
                expression: Box::new(expression.into_lir(symbol_table)),
                field: FieldIdentifier::Named(field),
            },
            Self::TupleElementAccess {
                expression,
                element_number,
                ..
            } => lir::Expr::FieldAccess {
                expression: Box::new(expression.into_lir(symbol_table)),
                field: FieldIdentifier::Unamed(element_number),
            },
            Self::Map {
                expression,
                function_expression,
                ..
            } => lir::Expr::Map {
                mapped: Box::new(expression.into_lir(symbol_table)),
                function: Box::new(function_expression.into_lir(symbol_table)),
            },
            Self::Fold {
                expression,
                initialization_expression,
                function_expression,
                ..
            } => lir::Expr::Fold {
                folded: Box::new(expression.into_lir(symbol_table)),
                initialization: Box::new(initialization_expression.into_lir(symbol_table)),
                function: Box::new(function_expression.into_lir(symbol_table)),
            },
            Self::Sort {
                expression,
                function_expression,
                ..
            } => lir::Expr::Sort {
                sorted: Box::new(expression.into_lir(symbol_table)),
                function: Box::new(function_expression.into_lir(symbol_table)),
            },
            Self::Zip { arrays, .. } => lir::Expr::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
        }
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            Self::Identifier { id, .. } => OtherOp::IfThenElse
                .to_string()
                .eq(symbol_table.get_name(*id)),
            _ => false,
        }
    }
}

impl IntoLir<&'_ SymbolTable> for hir::Expr {
    type Lir = lir::Expr;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        self.kind.into_lir(symbol_table)
    }
    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        self.kind.is_if_then_else(symbol_table)
    }
}
