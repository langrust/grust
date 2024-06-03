prelude! {
    ast::{ expr::Application, symbol::SymbolKind, stream },
}

use super::HIRFromAST;

impl HIRFromAST for stream::Expr {
    type HIR = hir::stream::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR stream expression and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
        let loc = &location;
        let kind = match self {
            stream::Expr::Application(Application {
                function_expression,
                inputs: inputs_stream_expressions,
            }) => match *function_expression {
                stream::Expr::Identifier(node) if symbol_table.is_node(&node, false) => {
                    let called_node_id =
                        symbol_table.get_node_id(&node, false, location.clone(), errors)?;
                    let node_symbol = symbol_table
                        .get_symbol(called_node_id)
                        .expect("there should be a symbol")
                        .clone();
                    match node_symbol.kind() {
                        SymbolKind::Node { inputs, .. } => {
                            // check inputs and node_inputs have the same length
                            if inputs.len() != inputs_stream_expressions.len() {
                                let error = Error::IncompatibleInputsNumber {
                                    given_inputs_number: inputs_stream_expressions.len(),
                                    expected_inputs_number: inputs.len(),
                                    location: location.clone(),
                                };
                                errors.push(error);
                                return Err(TerminationError);
                            }

                            hir::stream::Kind::NodeApplication {
                                calling_node_id: symbol_table.get_current_node_id(),
                                called_node_id,
                                inputs: inputs_stream_expressions
                                    .into_iter()
                                    .zip(inputs)
                                    .map(|(input, id)| {
                                        Ok((*id, input.clone().hir_from_ast(symbol_table, errors)?))
                                    })
                                    .collect::<TRes<Vec<_>>>()?,
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                function_expression => hir::stream::Kind::Expression {
                    expression: hir::expr::Kind::Application {
                        function_expression: Box::new(
                            function_expression.hir_from_ast(symbol_table, errors)?,
                        ),
                        inputs: inputs_stream_expressions
                            .into_iter()
                            .map(|input| input.clone().hir_from_ast(symbol_table, errors))
                            .collect::<TRes<Vec<_>>>()?,
                    },
                },
            },
            stream::Expr::Fby(stream::Fby {
                constant,
                expression,
            }) => {
                // check the constant expression is indeed constant
                constant.check_is_constant(symbol_table, errors)?;

                hir::stream::Kind::FollowedBy {
                    constant: Box::new(constant.hir_from_ast(symbol_table, errors)?),
                    expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
                }
            }
            stream::Expr::Constant(constant) => hir::stream::Kind::Expression {
                expression: hir::expr::Kind::Constant { constant },
            },
            stream::Expr::Identifier(id) => {
                let id = symbol_table
                    .get_identifier_id(&id, false, location.clone(), &mut vec![])
                    .or_else(|_| {
                        symbol_table.get_function_id(&id, false, location.clone(), errors)
                    })?;
                hir::stream::Kind::Expression {
                    expression: hir::expr::Kind::Identifier { id },
                }
            }
            stream::Expr::Unop(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Binop(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::IfThenElse(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::TypedAbstraction(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(&location, symbol_table, errors)?,
            },
            stream::Expr::Structure(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(&location, symbol_table, errors)?,
            },
            stream::Expr::Tuple(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Enumeration(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(&location, symbol_table, errors)?,
            },
            stream::Expr::Array(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Match(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::FieldAccess(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::TupleElementAccess(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Map(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Fold(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Sort(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
            stream::Expr::Zip(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(loc, symbol_table, errors)?,
            },
        };
        Ok(hir::stream::Expr {
            kind,
            typing: None,
            location,
            dependencies: hir::Dependencies::new(),
        })
    }
}
