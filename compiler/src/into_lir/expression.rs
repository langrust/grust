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
            Self::UnOp { op, expr } => {
                let expr = expr.into_lir(symbol_table);
                lir::Expr::unop(op, expr)
            }
            Self::BinOp { op, lft, rgt } => {
                let lft = lft.into_lir(symbol_table);
                let rgt = rgt.into_lir(symbol_table);
                lir::Expr::binop(op, lft, rgt)
            }
            Self::IfThenElse { cnd, thn, els } => {
                let cnd = cnd.into_lir(symbol_table);
                let thn = thn.into_lir(symbol_table);
                let els = els.into_lir(symbol_table);
                lir::Expr::ite(
                    cnd,
                    Block::new(vec![Stmt::ExprLast { expr: thn }]),
                    Block::new(vec![Stmt::ExprLast { expr: els }]),
                )
            }
            Self::Application { fun, inputs, .. } => {
                let arguments = inputs
                    .into_iter()
                    .map(|input| input.into_lir(symbol_table))
                    .collect();
                lir::Expr::FunctionCall {
                    function: Box::new(fun.into_lir(symbol_table)),
                    arguments,
                }
            }
            Self::Abstraction { inputs, expr, .. } => {
                let inputs = inputs
                    .iter()
                    .map(|id| {
                        (
                            symbol_table.get_name(*id).clone(),
                            symbol_table.get_typ(*id).clone(),
                        )
                    })
                    .collect();
                let output = expr.get_typ().expect("it should be typed").clone();
                lir::Expr::Lambda {
                    inputs,
                    output,
                    body: Box::new(lir::Expr::Block {
                        block: Block {
                            statements: vec![Stmt::ExprLast {
                                expr: expr.into_lir(symbol_table),
                            }],
                        },
                    }),
                }
            }
            Self::Structure { id, fields, .. } => lir::Expr::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, expr)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expr.into_lir(symbol_table),
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
            Self::Match { expr, arms, .. } => lir::Expr::Match {
                matched: Box::new(expr.into_lir(symbol_table)),
                arms: arms
                    .into_iter()
                    .map(|(pattern, guard, body, expr)| {
                        (
                            pattern.into_lir(symbol_table),
                            guard.map(|expr| expr.into_lir(symbol_table)),
                            if body.is_empty() {
                                expr.into_lir(symbol_table)
                            } else {
                                let mut statements = body
                                    .into_iter()
                                    .map(|statement| statement.into_lir(symbol_table))
                                    .collect::<Vec<_>>();
                                statements.push(Stmt::ExprLast {
                                    expr: expr.into_lir(symbol_table),
                                });
                                lir::Expr::Block {
                                    block: Block { statements },
                                }
                            },
                        )
                    })
                    .collect(),
            },
            Self::FieldAccess { expr, field, .. } => lir::Expr::FieldAccess {
                expr: Box::new(expr.into_lir(symbol_table)),
                field: FieldIdentifier::Named(field),
            },
            Self::TupleElementAccess {
                expr,
                element_number,
                ..
            } => lir::Expr::FieldAccess {
                expr: Box::new(expr.into_lir(symbol_table)),
                field: FieldIdentifier::Unnamed(element_number),
            },
            Self::Map { expr, fun, .. } => lir::Expr::Map {
                mapped: Box::new(expr.into_lir(symbol_table)),
                function: Box::new(fun.into_lir(symbol_table)),
            },
            Self::Fold {
                array, init, fun, ..
            } => lir::Expr::fold(
                array.into_lir(symbol_table),
                init.into_lir(symbol_table),
                fun.into_lir(symbol_table),
            ),
            Self::Sort { expr, fun, .. } => lir::Expr::Sort {
                sorted: Box::new(expr.into_lir(symbol_table)),
                function: Box::new(fun.into_lir(symbol_table)),
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
    fn get_typ(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        self.kind.is_if_then_else(symbol_table)
    }
}
