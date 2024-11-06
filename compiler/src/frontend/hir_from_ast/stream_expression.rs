prelude! {
    ast::{ expr::Application, symbol::SymbolKind, stream },
    itertools::Itertools,
}

impl<'a> HIRFromAST<hir::ctx::PatLoc<'a>> for stream::When {
    type HIR = hir::stream::Kind;

    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<Self::HIR> {
        let stream::When {
            pattern: event_pattern,
            guard,
            expression,
            ..
        } = self;
        let location = Location::default();
        let mut arms = vec![];

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use

        // create map from event_id to index in tuple pattern
        let (events_indices, events_nb, no_event_tuple) = {
            let mut events_indices = HashMap::with_capacity(arms.len());
            let mut idx = 0;
            event_pattern.place_events(&mut events_indices, &mut idx, ctxt.syms, ctxt.errors)?;
            // default event_pattern tuple
            let no_event_tuple: Vec<_> =
                std::iter::repeat(hir::pattern::init(hir::pattern::Kind::default()))
                    .take(idx)
                    .collect();
            (events_indices, idx, no_event_tuple)
        };

        // create action arm
        {
            ctxt.syms.local();

            // set local context + create matched pattern
            let (match_pattern, guard) = {
                let mut elements = no_event_tuple;
                let opt_rising_edges = event_pattern.create_tuple_pattern(
                    &mut elements,
                    &events_indices,
                    ctxt.syms,
                    ctxt.errors,
                )?;
                let matched = hir::pattern::init(hir::pattern::Kind::tuple(elements));

                // transform AST guard into HIR
                let mut guard = guard
                    .map(|expression| expression.hir_from_ast(ctxt))
                    .transpose()?;
                // add rising edge detection to the guard
                if let Some(rising_edges) = opt_rising_edges {
                    if let Some(old_guard) = guard.take() {
                        guard = Some(hir::stream::expr(hir::stream::Kind::expr(
                            hir::expr::Kind::binop(
                                operator::BinaryOperator::And,
                                old_guard,
                                rising_edges,
                            ),
                        )));
                    } else {
                        guard = Some(rising_edges)
                    }
                };

                (matched, guard)
            };
            // transform into HIR
            let expression = expression.hir_from_ast(ctxt)?;
            ctxt.syms.global();
            arms.push((match_pattern, guard, vec![], expression));
        }

        // create default arm
        {
            let match_pattern = hir::Pattern {
                kind: hir::pattern::Kind::Default,
                typing: None,
                location: location.clone(),
            };
            let pat = ctxt.pat.expect("there should be a pattern");
            // wraps events in 'none' and signals in 'fby'
            let expression = pat.into_default_expr(&HashMap::new(), ctxt.syms, ctxt.errors)?;
            arms.push((match_pattern, None, vec![], expression))
        }

        // construct the match expression
        let match_expr = {
            // create tuple expression to match
            let expr = {
                let elements = events_indices
                    .iter()
                    .sorted_by_key(|(_, idx)| **idx)
                    .map(|(event_id, _)| {
                        hir::stream::expr(hir::stream::Kind::expr(hir::expr::Kind::ident(
                            *event_id,
                        )))
                    })
                    .collect::<Vec<_>>();
                debug_assert!(elements.len() == events_nb);
                hir::stream::expr(hir::stream::Kind::expr(hir::expr::Kind::tuple(elements)))
            };
            // construct the match expression
            hir::stream::Kind::expr(hir::expr::Kind::match_expr(expr, arms))
        };

        Ok(match_expr)
    }
}

impl<'a> HIRFromAST<hir::ctx::PatLoc<'a>> for stream::Expr {
    type HIR = hir::stream::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR stream expression and check identifiers good use
    fn hir_from_ast(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<Self::HIR> {
        let kind = match self {
            stream::Expr::Application(Application {
                function_expression,
                inputs: inputs_stream_expressions,
            }) => match *function_expression {
                stream::Expr::Identifier(node) if ctxt.syms.is_node(&node, false) => {
                    let called_node_id =
                        ctxt.syms
                            .get_node_id(&node, false, ctxt.loc.clone(), ctxt.errors)?;
                    let node_symbol = ctxt
                        .syms
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
                                    location: ctxt.loc.clone(),
                                };
                                ctxt.errors.push(error);
                                return Err(TerminationError);
                            }

                            hir::stream::Kind::call(
                                called_node_id,
                                inputs_stream_expressions
                                    .into_iter()
                                    .zip(inputs)
                                    .map(|(input, id)| Ok((*id, input.clone().hir_from_ast(ctxt)?)))
                                    .collect::<TRes<Vec<_>>>()?,
                            )
                        }
                        _ => unreachable!(),
                    }
                }
                function_expression => hir::stream::Kind::Expression {
                    expression: hir::expr::Kind::Application {
                        function_expression: Box::new(function_expression.hir_from_ast(ctxt)?),
                        inputs: inputs_stream_expressions
                            .into_iter()
                            .map(|input| input.clone().hir_from_ast(ctxt))
                            .collect::<TRes<Vec<_>>>()?,
                    },
                },
            },
            stream::Expr::Last(stream::Last { ident, constant }) => {
                let default = hir::stream::Kind::Expression {
                    expression: hir::expr::Kind::constant(Constant::Default),
                };
                let constant = constant.map_or(Ok(hir::stream::expr(default)), |cst| {
                    // check the constant expression is indeed constant
                    cst.check_is_constant(ctxt.syms, ctxt.errors)?;
                    cst.hir_from_ast(ctxt)
                })?;

                let id = ctxt.syms.get_identifier_id(
                    &ident.to_string(),
                    false,
                    ctxt.loc.clone(),
                    ctxt.errors,
                )?;

                hir::stream::Kind::FollowedBy {
                    constant: Box::new(constant),
                    id,
                }
            }
            stream::Expr::Emit(stream::Emit { expr, .. }) => {
                hir::stream::Kind::some_event(expr.hir_from_ast(ctxt)?)
            }
            stream::Expr::Constant(constant) => hir::stream::Kind::Expression {
                expression: hir::expr::Kind::Constant { constant },
            },
            stream::Expr::Identifier(id) => {
                let id = ctxt
                    .syms
                    .get_identifier_id(&id, false, ctxt.loc.clone(), &mut vec![])
                    .or_else(|_| {
                        ctxt.syms
                            .get_function_id(&id, false, ctxt.loc.clone(), ctxt.errors)
                    })?;
                hir::stream::Kind::Expression {
                    expression: hir::expr::Kind::Identifier { id },
                }
            }
            stream::Expr::Unop(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Binop(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::IfThenElse(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::TypedAbstraction(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Structure(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Tuple(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Enumeration(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Array(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Match(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::FieldAccess(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::TupleElementAccess(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Map(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Fold(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Sort(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
            stream::Expr::Zip(expression) => hir::stream::Kind::Expression {
                expression: expression.hir_from_ast(ctxt)?,
            },
        };
        Ok(hir::stream::Expr {
            kind,
            typing: None,
            location: ctxt.loc.clone(),
            dependencies: hir::Dependencies::new(),
        })
    }
}

impl<'a> HIRFromAST<hir::ctx::PatLoc<'a>> for stream::ReactExpr {
    type HIR = hir::stream::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR stream expression and check identifiers good use
    fn hir_from_ast(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<Self::HIR> {
        match self {
            stream::ReactExpr::Expr(expr) => expr.hir_from_ast(ctxt),
            stream::ReactExpr::When(expr) => {
                let kind = expr.hir_from_ast(ctxt)?;
                Ok(hir::stream::Expr {
                    kind,
                    typing: None,
                    location: ctxt.loc.clone(),
                    dependencies: hir::Dependencies::new(),
                })
            }
        }
    }
}
