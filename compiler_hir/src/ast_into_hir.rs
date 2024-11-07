prelude! {
    itertools::Itertools,
}

/// AST transformation into HIR.
pub trait AstIntoHir<Ctx> {
    /// Corresponding HIR construct.
    type Hir;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut Ctx) -> TRes<Self::Hir>;
}

impl AstIntoHir<hir::ctx::Simple<'_>> for Ast {
    type Hir = hir::File;

    fn into_hir(self, ctx: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        // initialize symbol table with builtin operators
        ctx.symbols.initialize();

        // store elements in symbol table
        self.store(ctx.symbols, ctx.errors)?;

        let (typedefs, functions, components, imports, exports, services) =
            self.items.into_iter().fold(
                (vec![], vec![], vec![], vec![], vec![], vec![]),
                |(
                    mut typedefs,
                    mut functions,
                    mut components,
                    mut imports,
                    mut exports,
                    mut services,
                ),
                 item| {
                    match item {
                        ast::Item::Component(component) => components.push(component.into_hir(ctx)),
                        ast::Item::Function(function) => functions.push(function.into_hir(ctx)),
                        ast::Item::Typedef(typedef) => typedefs.push(typedef.into_hir(ctx)),
                        ast::Item::Service(service) => services.push(service.into_hir(ctx)),
                        ast::Item::Import(import) => imports.push(
                            import
                                .into_hir(ctx)
                                .map(|res| (ctx.symbols.get_fresh_id(), res)),
                        ),
                        ast::Item::Export(export) => exports.push(
                            export
                                .into_hir(ctx)
                                .map(|res| (ctx.symbols.get_fresh_id(), res)),
                        ),
                        ast::Item::ComponentImport(component) => {
                            components.push(component.into_hir(ctx))
                        }
                    }
                    (typedefs, functions, components, imports, exports, services)
                },
            );

        let interface = Interface {
            services: services.into_iter().collect::<TRes<Vec<_>>>()?,
            imports: imports.into_iter().collect::<TRes<_>>()?,
            exports: exports.into_iter().collect::<TRes<_>>()?,
        };

        Ok(hir::File {
            typedefs: typedefs.into_iter().collect::<TRes<Vec<_>>>()?,
            functions: functions.into_iter().collect::<TRes<Vec<_>>>()?,
            components: components.into_iter().collect::<TRes<Vec<_>>>()?,
            interface,
            loc: Location::default(),
        })
    }
}

mod interface_impl {
    prelude! {
        ast::interface::{
            TimeRange, FlowDeclaration, FlowExport, FlowImport,
            FlowInstantiation, FlowKind, FlowPattern, FlowStatement, Service,
        },
        hir::{
            interface::{
                FlowDeclaration as HIRFlowDeclaration, FlowExport as HIRFlowExport,
                FlowImport as HIRFlowImport, FlowInstantiation as HIRFlowInstantiation,
                FlowStatement as HIRFlowStatement,
            },
        },
    }

    impl<'a> AstIntoHir<hir::ctx::Simple<'a>> for Service {
        type Hir = hir::Service;

        fn into_hir(self, ctx: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let id = ctx.symbols.insert_service(
                self.ident.to_string(),
                true,
                Location::default(),
                ctx.errors,
            )?;

            let time_range = if let Some(TimeRange { min, max, .. }) = self.time_range {
                (min.base10_parse().unwrap(), max.base10_parse().unwrap())
            } else {
                (10, 500)
            };

            ctx.symbols.local();
            let statements = self
                .flow_statements
                .into_iter()
                .map(|flow_statement| {
                    flow_statement
                        .into_hir(ctx)
                        .map(|res| (ctx.symbols.get_fresh_id(), res))
                })
                .collect::<TRes<HashMap<_, _>>>()?;
            let graph = Default::default();
            ctx.symbols.global();

            Ok(hir::Service {
                id,
                time_range,
                statements,
                graph,
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Simple<'a>> for FlowImport {
        type Hir = HIRFlowImport;

        fn into_hir(mut self, ctx: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let loc = Location::default();

            let last = self.typed_path.left.segments.pop().unwrap().into_value();
            let name = last.ident.to_string();
            assert!(last.arguments.is_none());
            let path = self.typed_path.left;
            let flow_type = {
                let inner = self.typed_path.right.into_hir(&mut ctx.add_loc(&loc))?;
                match self.kind {
                    FlowKind::Signal(_) => Typ::signal(inner),
                    FlowKind::Event(_) => Typ::event(inner),
                }
            };
            let id = ctx.symbols.insert_flow(
                name,
                Some(path.clone()),
                self.kind,
                flow_type.clone(),
                true,
                loc.clone(),
                ctx.errors,
            )?;

            Ok(HIRFlowImport {
                import_token: self.import_token,
                id,
                path,
                colon_token: self.typed_path.colon,
                flow_type,
                semi_token: self.semi_token,
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Simple<'a>> for FlowExport {
        type Hir = HIRFlowExport;

        fn into_hir(mut self, ctx: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let loc = Location::default();

            let last = self.typed_path.left.segments.pop().unwrap().into_value();
            let name = last.ident.to_string();
            assert!(last.arguments.is_none());
            let path = self.typed_path.left;
            let flow_type = {
                let inner = self.typed_path.right.into_hir(&mut ctx.add_loc(&loc))?;
                match self.kind {
                    FlowKind::Signal(_) => Typ::signal(inner),
                    FlowKind::Event(_) => Typ::event(inner),
                }
            };
            let id = ctx.symbols.insert_flow(
                name,
                Some(path.clone()),
                self.kind,
                flow_type.clone(),
                true,
                loc.clone(),
                ctx.errors,
            )?;

            Ok(HIRFlowExport {
                export_token: self.export_token,
                id,
                path,
                colon_token: self.typed_path.colon,
                flow_type,
                semi_token: self.semi_token,
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Simple<'a>> for FlowStatement {
        type Hir = HIRFlowStatement;

        fn into_hir(self, ctx: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            match self {
                FlowStatement::Declaration(FlowDeclaration {
                    let_token,
                    typed_pattern,
                    eq_token,
                    expr,
                    semi_token,
                }) => {
                    let pattern = typed_pattern.into_hir(ctx)?;
                    let expr = expr.into_hir(&mut ctx.add_loc(&Location::default()))?;

                    Ok(HIRFlowStatement::Declaration(HIRFlowDeclaration {
                        let_token,
                        pattern,
                        eq_token,
                        expr,
                        semi_token,
                    }))
                }
                FlowStatement::Instantiation(FlowInstantiation {
                    pattern,
                    eq_token,
                    expr,
                    semi_token,
                }) => {
                    // transform pattern and check its identifiers exist
                    let pattern = pattern.into_hir(ctx)?;
                    // transform the expression
                    let expr = expr.into_hir(&mut ctx.add_loc(&Location::default()))?;

                    Ok(HIRFlowStatement::Instantiation(HIRFlowInstantiation {
                        pattern,
                        eq_token,
                        expr,
                        semi_token,
                    }))
                }
            }
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Simple<'a>> for FlowPattern {
        type Hir = hir::stmt::Pattern;

        fn into_hir(self, ctx: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let loc = Location::default();

            match self {
                FlowPattern::Single { ident } => {
                    let id = ctx.symbols.get_flow_id(
                        &ident.to_string(),
                        false,
                        loc.clone(),
                        ctx.errors,
                    )?;
                    let typing = ctx.symbols.get_type(id);

                    Ok(hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Identifier { id },
                        typing: Some(typing.clone()),
                        loc,
                    })
                }
                FlowPattern::SingleTyped {
                    kind, ident, ty, ..
                } => {
                    let inner = ty.into_hir(&mut ctx.add_loc(&loc))?;
                    let flow_typing = match kind {
                        FlowKind::Signal(_) => Typ::signal(inner),
                        FlowKind::Event(_) => Typ::event(inner),
                    };
                    let id = ctx.symbols.insert_flow(
                        ident.to_string(),
                        None,
                        kind,
                        flow_typing.clone(),
                        true,
                        loc.clone(),
                        ctx.errors,
                    )?;

                    Ok(hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Typed {
                            id,
                            typing: flow_typing.clone(),
                        },
                        typing: Some(flow_typing),
                        loc,
                    })
                }
                FlowPattern::Tuple { patterns, .. } => {
                    let elements = patterns
                        .into_iter()
                        .map(|pattern| pattern.into_hir(ctx))
                        .collect::<TRes<Vec<_>>>()?;

                    let mut types = elements
                        .iter()
                        .map(|pattern| pattern.typing.as_ref().unwrap().clone())
                        .collect::<Vec<_>>();
                    let ty = if types.len() == 1 {
                        types.pop().unwrap()
                    } else {
                        Typ::tuple(types)
                    };
                    Ok(hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Tuple { elements },
                        typing: Some(ty),
                        loc,
                    })
                }
            }
        }
    }
}

impl AstIntoHir<ctx::Simple<'_>> for ast::Function {
    type Hir = Function;

    // pre-condition: function and its inputs are already stored in symbol table
    // post-condition: construct HIR function and check identifiers good use
    fn into_hir(self, ctx: &mut ctx::Simple) -> TRes<Self::Hir> {
        let name = self.ident.to_string();
        let loc = Location::default();
        let id = ctx
            .symbols
            .get_function_id(&name, false, loc.clone(), ctx.errors)?;

        // create local context with all signals
        ctx.symbols.local();
        ctx.symbols.restore_context(id);

        // insert function output type in symbol table
        let output_typing = self.output_type.into_hir(&mut ctx.add_loc(&loc))?;
        if !self.contract.clauses.is_empty() {
            let _ = ctx.symbols.insert_function_result(
                output_typing.clone(),
                true,
                loc.clone(),
                ctx.errors,
            )?;
        }
        ctx.symbols.set_function_output_type(id, output_typing);

        let (statements, returned) = self.statements.into_iter().fold(
            (vec![], None),
            |(mut declarations, option_returned), statement| match statement {
                ast::Stmt::Declaration(declaration) => {
                    declarations.push(declaration.into_hir(ctx));
                    (declarations, option_returned)
                }
                ast::Stmt::Return(ast::stmt::Return { expression, .. }) => {
                    assert!(option_returned.is_none());
                    (
                        declarations,
                        Some(expression.into_hir(&mut ctx.add_pat_loc(None, &loc))),
                    )
                }
            },
        );
        let contract = self.contract.into_hir(ctx)?;

        ctx.symbols.global();

        Ok(hir::Function {
            id,
            contract,
            statements: statements.into_iter().collect::<TRes<Vec<_>>>()?,
            returned: returned.unwrap()?,
            loc,
        })
    }
}

impl AstIntoHir<ctx::Simple<'_>> for ast::Component {
    type Hir = hir::Component;

    // pre-condition: node and its signals are already stored in symbol table
    // post-condition: construct HIR node and check identifiers good use
    fn into_hir(self, ctx: &mut ctx::Simple) -> TRes<Self::Hir> {
        let name = self.ident.to_string();
        let loc = Location::default();
        let id = ctx
            .symbols
            .get_node_id(&name, false, loc.clone(), ctx.errors)?;

        // create local context with all signals
        ctx.symbols.local();
        ctx.symbols.restore_context(id);

        let statements = self
            .equations
            .into_iter()
            .map(|equation| equation.into_hir(ctx))
            .collect::<TRes<Vec<_>>>()?;
        let contract = self.contract.into_hir(ctx)?;

        ctx.symbols.global();

        Ok(hir::Component::Definition(hir::ComponentDefinition {
            id,
            statements,
            contract,
            loc,
            graph: graph::DiGraphMap::new(),
            reduced_graph: graph::DiGraphMap::new(),
            memory: hir::Memory::new(),
        }))
    }
}

impl AstIntoHir<ctx::Simple<'_>> for ast::ComponentImport {
    type Hir = hir::Component;

    // pre-condition: node and its signals are already stored in symbol table
    // post-condition: construct HIR node
    fn into_hir(self, ctx: &mut ctx::Simple) -> TRes<Self::Hir> {
        let last = self.path.clone().segments.pop().unwrap().into_value();
        let name = last.ident.to_string();
        assert!(last.arguments.is_none());

        let loc = Location::default();
        let id = ctx
            .symbols
            .get_node_id(&name, false, loc.clone(), ctx.errors)?;

        Ok(hir::Component::Import(hir::ComponentImport {
            id,
            path: self.path,
            loc,
            graph: graph::DiGraphMap::new(),
        }))
    }
}

mod flow_expr_impl {
    prelude! {
        ast::interface::{
            FlowExpression, ComponentCall, OnChange, Merge,
            Scan, Throttle, Timeout,
        },
        hir::flow,
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for ast::interface::Sample {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Sample {
                expr: Box::new(self.expr.into_hir(ctx)?),
                period_ms: self.period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for Scan {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Scan {
                expr: Box::new(self.expr.into_hir(ctx)?),
                period_ms: self.period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for Timeout {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Timeout {
                expr: Box::new(self.expr.into_hir(ctx)?),
                deadline: self.deadline.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for Throttle {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Throttle {
                expr: Box::new(self.expr.into_hir(ctx)?),
                delta: self.delta,
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for OnChange {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::OnChange {
                expr: Box::new(self.expr.into_hir(ctx)?),
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for Merge {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Merge {
                expr_1: Box::new(self.expr_1.into_hir(ctx)?),
                expr_2: Box::new(self.expr_2.into_hir(ctx)?),
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for ComponentCall {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let name = self.ident_component.to_string();

            // get called component id
            let component_id =
                ctx.symbols
                    .get_node_id(&name, false, ctx.loc.clone(), ctx.errors)?;

            let component_inputs = ctx.symbols.get_node_inputs(component_id).clone();

            // check inputs and node_inputs have the same length
            if self.inputs.len() != component_inputs.len() {
                let error = Error::ArityMismatch {
                    input_count: self.inputs.len(),
                    arity: component_inputs.len(),
                    loc: ctx.loc.clone(),
                };
                ctx.errors.push(error);
                return Err(TerminationError);
            }

            // transform inputs and map then to the identifiers of the component inputs
            let inputs = self
                .inputs
                .into_iter()
                .zip(component_inputs)
                .map(|(input, id)| Ok((id, input.into_hir(ctx)?)))
                .collect::<TRes<Vec<_>>>()?;

            Ok(hir::flow::Kind::ComponentCall {
                component_id,
                inputs,
            })
        }
    }

    impl<'a> AstIntoHir<hir::ctx::Loc<'a>> for FlowExpression {
        type Hir = flow::Expr;

        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression and check identifiers good use
        fn into_hir(self, ctx: &mut hir::ctx::Loc<'a>) -> TRes<Self::Hir> {
            let kind = match self {
                FlowExpression::Ident(ident) => {
                    let id = ctx
                        .symbols
                        .get_flow_id(&ident, false, ctx.loc.clone(), ctx.errors)?;
                    flow::Kind::Ident { id }
                }
                FlowExpression::ComponentCall(expr) => expr.into_hir(ctx)?,
                FlowExpression::Sample(expr) => expr.into_hir(ctx)?,
                FlowExpression::Scan(expr) => expr.into_hir(ctx)?,
                FlowExpression::Timeout(expr) => expr.into_hir(ctx)?,
                FlowExpression::Throttle(expr) => expr.into_hir(ctx)?,
                FlowExpression::OnChange(expr) => expr.into_hir(ctx)?,
                FlowExpression::Merge(expr) => expr.into_hir(ctx)?,
            };
            Ok(flow::Expr {
                kind,
                typing: None,
                loc: ctx.loc.clone(),
            })
        }
    }
}

impl AstIntoHir<ctx::Simple<'_>> for ast::Contract {
    type Hir = Contract;

    fn into_hir(self, ctx: &mut ctx::Simple) -> TRes<Self::Hir> {
        use ast::contract::ClauseKind;
        let (requires, ensures, invariant) = self.clauses.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut requires, mut ensures, mut invariant), clause| {
                match clause.kind {
                    ClauseKind::Requires(_) => requires.push(clause.term.into_hir(ctx)),
                    ClauseKind::Ensures(_) => ensures.push(clause.term.into_hir(ctx)),
                    ClauseKind::Invariant(_) => invariant.push(clause.term.into_hir(ctx)),
                    ClauseKind::Assert(_) => todo!(),
                };
                (requires, ensures, invariant)
            },
        );

        Ok(hir::Contract {
            requires: requires.into_iter().collect::<TRes<Vec<_>>>()?,
            ensures: ensures.into_iter().collect::<TRes<Vec<_>>>()?,
            invariant: invariant.into_iter().collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<'a> AstIntoHir<ctx::Simple<'a>> for ast::contract::Term {
    type Hir = hir::contract::Term;

    fn into_hir(self, ctx: &mut ctx::Simple<'a>) -> TRes<Self::Hir> {
        use ast::contract::*;
        let loc = Location::default();
        match self {
            Term::Result(_) => {
                let id = ctx
                    .symbols
                    .get_function_result_id(false, loc.clone(), ctx.errors)?;
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::ident(id),
                    None,
                    loc,
                ))
            }
            Term::Implication(Implication { left, right, .. }) => {
                let left = left.into_hir(ctx)?;
                let right = right.into_hir(ctx)?;

                Ok(hir::contract::Term::new(
                    hir::contract::Kind::implication(left, right),
                    None,
                    loc,
                ))
            }
            Term::Enumeration(Enumeration {
                enum_name,
                elem_name,
            }) => {
                let enum_id =
                    ctx.symbols
                        .get_enum_id(&enum_name, false, loc.clone(), ctx.errors)?;
                let element_id = ctx.symbols.get_enum_elem_id(
                    &elem_name,
                    &enum_name,
                    false,
                    loc.clone(),
                    ctx.errors,
                )?;
                // TODO check elem is in enum
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::enumeration(enum_id, element_id),
                    None,
                    loc,
                ))
            }
            Term::Unary(Unary { op, term }) => Ok(hir::contract::Term::new(
                hir::contract::Kind::unary(op, term.into_hir(ctx)?),
                None,
                loc,
            )),
            Term::Binary(Binary { op, left, right }) => Ok(hir::contract::Term::new(
                hir::contract::Kind::binary(op, left.into_hir(ctx)?, right.into_hir(ctx)?),
                None,
                loc,
            )),
            Term::Constant(constant) => Ok(hir::contract::Term::new(
                hir::contract::Kind::constant(constant),
                None,
                loc,
            )),
            Term::Identifier(ident) => {
                let id = ctx
                    .symbols
                    .get_identifier_id(&ident, false, loc.clone(), ctx.errors)?;
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::ident(id),
                    None,
                    loc,
                ))
            }
            Term::ForAll(ForAll {
                ident, ty, term, ..
            }) => {
                let ty = ty.into_hir(&mut ctx.add_loc(&loc))?;
                ctx.symbols.local();
                let id = ctx.symbols.insert_identifier(
                    ident.clone(),
                    Some(ty),
                    true,
                    loc.clone(),
                    ctx.errors,
                )?;
                let term = term.into_hir(ctx)?;
                ctx.symbols.global();
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::forall(id, term),
                    None,
                    loc,
                ))
            }
            Term::EventImplication(EventImplication {
                pattern,
                event,
                term,
                ..
            }) => {
                // get the event identifier
                let event_id =
                    ctx.symbols
                        .get_identifier_id(&event, false, loc.clone(), ctx.errors)?;
                ctx.symbols.local();
                // set pattern signal in local context
                let pattern_id = ctx.symbols.insert_identifier(
                    pattern.clone(),
                    None,
                    true,
                    loc.clone(),
                    ctx.errors,
                )?;
                // transform term into HIR
                let right = term.into_hir(ctx)?;
                ctx.symbols.global();
                // construct right side of implication: `PresentEvent(pat) == event`
                let left = hir::contract::Term::new(
                    hir::contract::Kind::binary(
                        BOp::Eq,
                        hir::contract::Term::new(
                            hir::contract::Kind::present(event_id, pattern_id),
                            None,
                            loc.clone(),
                        ),
                        hir::contract::Term::new(
                            hir::contract::Kind::ident(event_id),
                            None,
                            loc.clone(),
                        ),
                    ),
                    None,
                    loc.clone(),
                );
                // construct result term
                // - `when pat = e? => t`
                // becomes
                // - `forall pat, PresentEvent(pat) == event => t`
                let term = hir::contract::Term::new(
                    hir::contract::Kind::forall(
                        pattern_id,
                        hir::contract::Term::new(
                            hir::contract::Kind::implication(left, right),
                            None,
                            loc.clone(),
                        ),
                    ),
                    None,
                    loc,
                );
                Ok(term)
            }
        }
    }
}

impl AstIntoHir<ctx::Simple<'_>> for ast::Eq {
    type Hir = hir::stream::Stmt;

    /// Pre-condition: equation's signal is already stored in symbol table.
    ///
    /// Post-condition: construct HIR equation and check identifiers good use.
    fn into_hir(self, ctx: &mut ctx::Simple) -> TRes<Self::Hir> {
        use ast::{
            equation::{Arm, Eq, Instantiation, Match},
            stmt::LetDecl,
        };
        let loc = Location::default();

        // get signals defined by the equation
        let mut defined_signals = HashMap::new();
        self.get_signals(&mut defined_signals, ctx.symbols, ctx.errors)?;

        match self {
            Eq::LocalDef(LetDecl {
                expr,
                typed_pattern: pattern,
                ..
            })
            | Eq::OutputDef(Instantiation { expr, pattern, .. }) => {
                let expr = expr.into_hir(&mut ctx.add_pat_loc(Some(&pattern), &loc))?;
                let pattern = pattern.into_hir(&mut ctx.add_loc(&loc))?;
                Ok(hir::Stmt { pattern, expr, loc })
            }
            Eq::Match(Match { expr, arms, .. }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().into_hir(&mut ctx.add_loc(&loc)))
                        .collect::<TRes<Vec<_>>>()?;
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        hir::stmt::init(hir::stmt::Kind::tuple(elements))
                    }
                };

                // for each arm, construct pattern guard and statements
                let arms = arms
                    .into_iter()
                    .map(
                        |Arm {
                             pattern,
                             guard,
                             equations,
                             ..
                         }| {
                            // transform pattern guard and equations into HIR
                            let (signals, pattern, guard, statements) = {
                                ctx.symbols.local();

                                // set local context: pattern signals + equations' signals
                                pattern.store(ctx.symbols, ctx.errors)?;
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctx.symbols,
                                            ctx.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform pattern guard and equations into HIR with local context
                                let pattern = pattern.into_hir(&mut ctx.add_loc(&loc))?;
                                let guard = guard
                                    .map(|(_, expression)| {
                                        expression.into_hir(&mut ctx.add_pat_loc(None, &loc))
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| equation.into_hir(ctx))
                                    .collect::<TRes<Vec<_>>>()?;

                                ctx.symbols.global();

                                (signals, pattern, guard, statements)
                            };

                            // create the tuple expression
                            let expression = {
                                // check defined signals are all the same
                                if defined_signals.len() != signals.len() {
                                    let error = Error::IncompatibleMatchStatements {
                                        expected: defined_signals.len(),
                                        received: signals.len(),
                                        loc: loc.clone(),
                                    };
                                    ctx.errors.push(error);
                                    return Err(TerminationError);
                                }
                                let mut elements = defined_signals
                                    .keys()
                                    .map(|signal_name| {
                                        if let Some(id) = signals.get(signal_name) {
                                            Ok(hir::stream::expr(hir::stream::Kind::expr(
                                                hir::expr::Kind::ident(*id),
                                            )))
                                        } else {
                                            let error = Error::MissingMatchStatement {
                                                identifier: signal_name.clone(),
                                                loc: loc.clone(),
                                            };
                                            ctx.errors.push(error);
                                            return Err(TerminationError);
                                        }
                                    })
                                    .collect::<TRes<Vec<_>>>()?;

                                // create the tuple expression
                                if elements.len() == 1 {
                                    elements.pop().unwrap()
                                } else {
                                    stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(
                                        elements,
                                    )))
                                }
                            };

                            Ok((pattern, guard, statements, expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                // construct the match expression
                let expr = stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                    expr.into_hir(&mut ctx.add_pat_loc(None, &loc))?,
                    arms,
                )));

                Ok(hir::Stmt { pattern, expr, loc })
            }
        }
    }
}

impl AstIntoHir<ctx::Simple<'_>> for ast::ReactEq {
    type Hir = stream::Stmt;

    /// Pre-condition: equation's signal is already stored in symbol table.
    ///
    /// Post-condition: construct HIR equation and check identifiers good use.
    fn into_hir(self, ctx: &mut ctx::Simple) -> TRes<Self::Hir> {
        use ast::{
            equation::{Arm, EventArmWhen, Instantiation, MatchWhen},
            stmt::LetDecl,
            ReactEq,
        };
        let loc = Location::default();

        // get signals defined by the equation
        let mut defined_signals = HashMap::new();
        self.get_signals(&mut defined_signals, ctx.symbols, ctx.errors)?;

        match self {
            ReactEq::LocalDef(LetDecl {
                expr,
                typed_pattern: pattern,
                ..
            })
            | ReactEq::OutputDef(Instantiation { expr, pattern, .. }) => {
                let expr = expr.into_hir(&mut ctx.add_pat_loc(Some(&pattern), &loc))?;
                let pattern = pattern.into_hir(&mut ctx.add_loc(&loc))?;
                Ok(hir::Stmt { pattern, expr, loc })
            }
            ReactEq::Match(ast::equation::Match { expr, arms, .. }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().into_hir(&mut ctx.add_loc(&loc)))
                        .collect::<TRes<Vec<_>>>()?;
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        hir::stmt::init(hir::stmt::Kind::tuple(elements))
                    }
                };

                // for each arm, construct pattern guard and statements
                let arms = arms
                    .into_iter()
                    .map(
                        |Arm {
                             pattern,
                             guard,
                             equations,
                             ..
                         }| {
                            // transform pattern guard and equations into HIR
                            let (signals, pattern, guard, statements) = {
                                ctx.symbols.local();

                                // set local context: pattern signals + equations' signals
                                pattern.store(ctx.symbols, ctx.errors)?;
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctx.symbols,
                                            ctx.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform pattern guard and equations into HIR with local context
                                let pattern = pattern.into_hir(&mut ctx.add_loc(&loc))?;
                                let guard = guard
                                    .map(|(_, expr)| {
                                        expr.into_hir(&mut ctx.add_pat_loc(None, &loc))
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| equation.into_hir(ctx))
                                    .collect::<TRes<Vec<_>>>()?;

                                ctx.symbols.global();

                                (signals, pattern, guard, statements)
                            };

                            // create the tuple expression
                            let expr = {
                                // check defined signals are all the same
                                if defined_signals.len() != signals.len() {
                                    let error = Error::IncompatibleMatchStatements {
                                        expected: defined_signals.len(),
                                        received: signals.len(),
                                        loc: loc.clone(),
                                    };
                                    ctx.errors.push(error);
                                    return Err(TerminationError);
                                }
                                let mut elements = defined_signals
                                    .keys()
                                    .map(|signal_name| {
                                        if let Some(id) = signals.get(signal_name) {
                                            Ok(stream::expr(stream::Kind::expr(
                                                hir::expr::Kind::ident(*id),
                                            )))
                                        } else {
                                            let error = Error::MissingMatchStatement {
                                                identifier: signal_name.clone(),
                                                loc: loc.clone(),
                                            };
                                            ctx.errors.push(error);
                                            return Err(TerminationError);
                                        }
                                    })
                                    .collect::<TRes<Vec<_>>>()?;

                                // create the tuple expression
                                if elements.len() == 1 {
                                    elements.pop().unwrap()
                                } else {
                                    stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(
                                        elements,
                                    )))
                                }
                            };

                            Ok((pattern, guard, statements, expr))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                // construct the match expression
                let expr = stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                    expr.into_hir(&mut ctx.add_pat_loc(None, &loc))?,
                    arms,
                )));

                Ok(hir::Stmt { pattern, expr, loc })
            }
            ReactEq::MatchWhen(MatchWhen { arms, .. }) => {
                // create the receiving pattern for the equation
                let defined_pattern = {
                    let mut elements = defined_signals.into_values().collect::<Vec<_>>();
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        ast::stmt::Pattern::tuple(ast::stmt::Tuple::new(elements))
                    }
                };

                let (
                    // map from event_id to index in tuple pattern
                    events_indices,
                    // default tuple pattern
                    default_pattern,
                ) = {
                    // create map from event_id to index in tuple pattern
                    let mut events_indices = HashMap::with_capacity(arms.len());
                    let mut idx = 0;
                    arms.iter()
                        .map(|event_arm| {
                            event_arm.pattern.place_events(
                                &mut events_indices,
                                &mut idx,
                                ctx.symbols,
                                ctx.errors,
                            )
                        })
                        .collect::<TRes<()>>()?;

                    // default event_pattern tuple
                    let default_pattern: Vec<_> =
                        std::iter::repeat(pattern::init(pattern::Kind::default()))
                            .take(idx)
                            .collect();

                    (events_indices, default_pattern)
                };

                // default arm
                let default = {
                    // create tuple pattern
                    let elements = default_pattern.clone();
                    let pattern = pattern::init(pattern::Kind::tuple(elements));
                    // transform guard and equations into HIR with local context
                    let guard = None;
                    // create the tuple expression
                    let expression = defined_pattern.into_default_expr(
                        &HashMap::new(),
                        ctx.symbols,
                        ctx.errors,
                    )?;

                    (pattern, guard, vec![], expression)
                };

                // for each arm construct hir pattern, guard and statements
                let mut match_arms = arms
                    .into_iter()
                    .map(
                        |EventArmWhen {
                             pattern: event_pattern,
                             guard,
                             equations,
                             ..
                         }| {
                            // transform event_pattern guard and equations into HIR
                            ctx.symbols.local();

                            // set local context + create matched pattern
                            let (matched_pattern, guard) = {
                                let mut elements = default_pattern.clone();
                                let opt_rising_edges = event_pattern.create_tuple_pattern(
                                    &mut elements,
                                    &events_indices,
                                    ctx.symbols,
                                    ctx.errors,
                                )?;
                                let matched = pattern::init(pattern::Kind::tuple(elements));

                                // transform AST guard into HIR
                                let mut guard = guard
                                    .map(|(_, expression)| {
                                        expression.into_hir(&mut ctx.add_pat_loc(None, &loc))
                                    })
                                    .transpose()?;
                                // add rising edge detection to the guard
                                if let Some(rising_edges) = opt_rising_edges {
                                    if let Some(old_guard) = guard.take() {
                                        guard = Some(hir::stream::expr(hir::stream::Kind::expr(
                                            hir::expr::Kind::binop(
                                                BOp::And,
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

                            // set and get local context: equations' signals
                            let signals = {
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctx.symbols,
                                            ctx.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;
                                signals
                            };

                            // transform equations into HIR with local context
                            let statements = equations
                                .into_iter()
                                .map(|equation| equation.into_hir(ctx))
                                .collect::<TRes<Vec<_>>>()?;

                            ctx.symbols.global();

                            // create the tuple expression
                            let expression = defined_pattern.into_default_expr(
                                &signals,
                                ctx.symbols,
                                ctx.errors,
                            )?;

                            Ok((matched_pattern, guard, statements, expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;
                match_arms.push(default);

                // construct the match expression
                let match_expr = {
                    // create tuple expression to match
                    let tuple_expr = {
                        let elements = events_indices
                            .iter()
                            .sorted_by_key(|(_, idx)| **idx)
                            .map(|(event_id, _)| {
                                stream::expr(stream::Kind::expr(hir::expr::Kind::ident(*event_id)))
                            })
                            .collect::<Vec<_>>();
                        stream::expr(stream::Kind::expr(hir::expr::Kind::tuple(elements)))
                    };
                    stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                        tuple_expr, match_arms,
                    )))
                };

                let pattern = defined_pattern.into_hir(&mut ctx.add_loc(&loc))?;

                Ok(hir::Stmt {
                    pattern,
                    expr: match_expr,
                    loc,
                })
            }
        }
    }
}

impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for ast::expr::UnOp<E>
where
    E: AstIntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::unop(self.op, self.expr.into_hir(ctx)?))
    }
}

impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Binop<E>
where
    E: AstIntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::binop(
            self.op,
            self.lft.into_hir(ctx)?,
            self.rgt.into_hir(ctx)?,
        ))
    }
}

impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for ast::expr::IfThenElse<E>
where
    E: AstIntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::if_then_else(
            self.cnd.into_hir(ctx)?,
            self.thn.into_hir(ctx)?,
            self.els.into_hir(ctx)?,
        ))
    }
}

impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Application<E>
where
    E: AstIntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::app(
            self.fun.into_hir(ctx)?,
            self.inputs
                .into_iter()
                .map(|input| input.into_hir(ctx))
                .collect::<TRes<Vec<_>>>()?,
        ))
    }
}

impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Structure<E>
where
    E: AstIntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression kind and check identifiers good use
        let id = ctx
            .symbols
            .get_struct_id(&self.name, false, ctx.loc.clone(), ctx.errors)?;
        let mut field_ids = ctx
            .symbols
            .get_struct_fields(id)
            .clone()
            .into_iter()
            .map(|id| (ctx.symbols.get_name(id).clone(), id))
            .collect::<HashMap<_, _>>();

        let fields = self
            .fields
            .into_iter()
            .map(|(field_name, expression)| {
                let id = field_ids.remove(&field_name).map_or_else(
                    || {
                        let error = Error::UnknownField {
                            structure_name: self.name.clone(),
                            field_name: field_name.clone(),
                            loc: ctx.loc.clone(),
                        };
                        ctx.errors.push(error);
                        Err(TerminationError)
                    },
                    |id| Ok(id),
                )?;
                let expression = expression.into_hir(ctx)?;
                Ok((id, expression))
            })
            .collect::<TRes<Vec<_>>>()?;

        // check if there are no missing fields
        field_ids
            .keys()
            .map(|field_name| {
                let error = Error::MissingField {
                    structure_name: self.name.clone(),
                    field_name: field_name.clone(),
                    loc: ctx.loc.clone(),
                };
                ctx.errors.push(error);
                Err::<(), _>(TerminationError)
            })
            .collect::<TRes<Vec<_>>>()?;

        Ok(expr::Kind::Structure { id, fields })
    }
}

mod simple_expr_impl {
    prelude! {
        ast::expr::*,
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Enumeration<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>>
        where
            E: AstIntoHir<hir::ctx::PatLoc<'a>>,
        {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            let enum_id =
                ctx.symbols
                    .get_enum_id(&self.enum_name, false, ctx.loc.clone(), ctx.errors)?;
            let elem_id = ctx.symbols.get_enum_elem_id(
                &self.elem_name,
                &self.enum_name,
                false,
                ctx.loc.clone(),
                ctx.errors,
            )?;
            // TODO check elem is in enum
            Ok(expr::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Array<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Array {
                elements: self
                    .elements
                    .into_iter()
                    .map(|expression| expression.into_hir(ctx))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Tuple<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::tuple(
                self.elements
                    .into_iter()
                    .map(|expression| expression.into_hir(ctx))
                    .collect::<TRes<Vec<_>>>()?,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Match<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::match_expr(
                self.expr.into_hir(ctx)?,
                self.arms
                    .into_iter()
                    .map(|arm| {
                        ctx.symbols.local();
                        arm.pattern.store(ctx.symbols, ctx.errors)?;
                        let pattern = arm.pattern.into_hir(&mut ctx.remove_pat())?;
                        let guard = arm.guard.map(|expr| expr.into_hir(ctx)).transpose()?;
                        let expr = arm.expr.into_hir(ctx)?;
                        ctx.symbols.global();
                        Ok((pattern, guard, vec![], expr))
                    })
                    .collect::<TRes<Vec<_>>>()?,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for FieldAccess<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::field_access(
                self.expr.into_hir(ctx)?,
                self.field,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for TupleElementAccess<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::tuple_access(
                self.expr.into_hir(ctx)?,
                self.element_number,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Map<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::map(
                self.expr.into_hir(ctx)?,
                self.fun.into_hir(ctx)?,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Fold<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::fold(
                self.array.into_hir(ctx)?,
                self.init.into_hir(ctx)?,
                self.fun.into_hir(ctx)?,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Sort<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::sort(
                self.expr.into_hir(ctx)?,
                self.fun.into_hir(ctx)?,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for Zip<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::zip(
                self.arrays
                    .into_iter()
                    .map(|array| array.into_hir(ctx))
                    .collect::<TRes<Vec<_>>>()?,
            ))
        }
    }

    impl<'a, E> AstIntoHir<hir::ctx::PatLoc<'a>> for TypedAbstraction<E>
    where
        E: AstIntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            ctx.symbols.local();
            let inputs = self
                .inputs
                .into_iter()
                .map(|(input_name, typing)| {
                    let typing = typing.into_hir(&mut ctx.remove_pat())?;
                    ctx.symbols.insert_identifier(
                        input_name,
                        Some(typing),
                        true,
                        ctx.loc.clone(),
                        ctx.errors,
                    )
                })
                .collect::<TRes<Vec<_>>>()?;
            let expr = self.expr.into_hir(ctx)?;
            ctx.symbols.global();

            Ok(expr::Kind::lambda(inputs, expr))
        }
    }

    impl<'a> AstIntoHir<hir::ctx::PatLoc<'a>> for ast::Expr {
        type Hir = hir::Expr;

        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR expression and check identifiers good use
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc<'a>) -> TRes<Self::Hir> {
            use ast::Expr::*;
            let kind = match self {
                Constant(constant) => hir::expr::Kind::Constant { constant },
                Identifier(id) => {
                    let id = ctx
                        .symbols
                        .get_identifier_id(&id, false, ctx.loc.clone(), &mut vec![])
                        .or_else(|_| {
                            ctx.symbols
                                .get_function_id(&id, false, ctx.loc.clone(), ctx.errors)
                        })?;
                    hir::expr::Kind::Identifier { id }
                }
                UnOp(e) => e.into_hir(ctx)?,
                Binop(e) => e.into_hir(ctx)?,
                IfThenElse(e) => e.into_hir(ctx)?,
                Application(e) => e.into_hir(ctx)?,
                TypedAbstraction(e) => e.into_hir(ctx)?,
                Structure(e) => e.into_hir(ctx)?,
                Tuple(e) => e.into_hir(ctx)?,
                Enumeration(e) => e.into_hir(ctx)?,
                Array(e) => e.into_hir(ctx)?,
                Match(e) => e.into_hir(ctx)?,
                FieldAccess(e) => e.into_hir(ctx)?,
                TupleElementAccess(e) => e.into_hir(ctx)?,
                Map(e) => e.into_hir(ctx)?,
                Fold(e) => e.into_hir(ctx)?,
                Sort(e) => e.into_hir(ctx)?,
                Zip(e) => e.into_hir(ctx)?,
            };
            Ok(hir::Expr {
                kind,
                typing: None,
                loc: ctx.loc.clone(),
                dependencies: Dependencies::new(),
            })
        }
    }
}

mod expr_pattern_impl {
    prelude! {
        ast::expr::{PatEnumeration, PatStructure, PatTuple},
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for PatStructure {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let id = ctx
                .symbols
                .get_struct_id(&self.name, false, ctx.loc.clone(), ctx.errors)?;
            let mut field_ids = ctx
                .symbols
                .get_struct_fields(id)
                .clone()
                .into_iter()
                .map(|id| (ctx.symbols.get_name(id).clone(), id))
                .collect::<HashMap<_, _>>();

            let fields = self
                .fields
                .into_iter()
                .map(|(field_name, optional_pattern)| {
                    let id = field_ids.remove(&field_name).map_or_else(
                        || {
                            let error = Error::UnknownField {
                                structure_name: self.name.clone(),
                                field_name: field_name.clone(),
                                loc: ctx.loc.clone(),
                            };
                            ctx.errors.push(error);
                            Err(TerminationError)
                        },
                        |id| Ok(id),
                    )?;
                    let pattern = optional_pattern
                        .map(|pattern| pattern.into_hir(ctx))
                        .transpose()?;
                    Ok((id, pattern))
                })
                .collect::<TRes<Vec<_>>>()?;

            if self.rest.is_none() {
                // check if there are no missing fields
                field_ids
                    .keys()
                    .map(|field_name| {
                        let error = Error::MissingField {
                            structure_name: self.name.clone(),
                            field_name: field_name.clone(),
                            loc: ctx.loc.clone(),
                        };
                        ctx.errors.push(error);
                        TRes::<()>::Err(TerminationError)
                    })
                    .collect::<TRes<Vec<_>>>()?;
            }

            Ok(hir::pattern::Kind::Structure { id, fields })
        }
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for PatEnumeration {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let enum_id =
                ctx.symbols
                    .get_enum_id(&self.enum_name, false, ctx.loc.clone(), ctx.errors)?;
            let elem_id = ctx.symbols.get_enum_elem_id(
                &self.elem_name,
                &self.enum_name,
                false,
                ctx.loc.clone(),
                ctx.errors,
            )?;
            Ok(hir::pattern::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for PatTuple {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            Ok(hir::pattern::Kind::tuple(
                self.elements
                    .into_iter()
                    .map(|pattern| pattern.into_hir(ctx))
                    .collect::<TRes<Vec<_>>>()?,
            ))
        }
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for ast::expr::Pattern {
        type Hir = hir::Pattern;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<Self::Hir> {
            use ast::expr::Pattern::*;
            let kind = match self {
                Constant(constant) => hir::pattern::Kind::Constant { constant },
                Identifier(name) => {
                    let id =
                        ctx.symbols
                            .get_identifier_id(&name, false, ctx.loc.clone(), ctx.errors)?;
                    hir::pattern::Kind::Identifier { id }
                }
                Structure(pat) => pat.into_hir(ctx)?,
                Enumeration(pat) => pat.into_hir(ctx)?,
                Tuple(pat) => pat.into_hir(ctx)?,
                // None => hir::pattern::Kind::None,
                Default => hir::pattern::Kind::Default,
            };

            Ok(hir::Pattern {
                kind,
                typing: None,
                loc: ctx.loc.clone(),
            })
        }
    }
}

mod stmt_pattern_impl {
    prelude! {
        ast::stmt::{Typed, Tuple},
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for Typed {
        type Hir = hir::stmt::Kind;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<hir::stmt::Kind> {
            let id = ctx.symbols.get_identifier_id(
                &self.ident.to_string(),
                false,
                ctx.loc.clone(),
                ctx.errors,
            )?;
            let typing = self.typing.into_hir(ctx)?;
            Ok(hir::stmt::Kind::Typed { id, typing })
        }
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for Tuple {
        type Hir = hir::stmt::Kind;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<hir::stmt::Kind> {
            Ok(hir::stmt::Kind::tuple(
                self.elements
                    .into_iter()
                    .map(|pattern| pattern.into_hir(ctx))
                    .collect::<TRes<Vec<_>>>()?,
            ))
        }
    }

    impl AstIntoHir<hir::ctx::Loc<'_>> for ast::stmt::Pattern {
        type Hir = hir::stmt::Pattern;

        fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<Self::Hir> {
            use ast::stmt::Pattern::*;
            let kind = match self {
                Identifier(ident) => {
                    let id = ctx.symbols.get_identifier_id(
                        &ident.to_string(),
                        false,
                        ctx.loc.clone(),
                        ctx.errors,
                    )?;
                    hir::stmt::Kind::Identifier { id }
                }
                Typed(pattern) => pattern.into_hir(ctx)?,
                Tuple(pattern) => pattern.into_hir(ctx)?,
            };

            Ok(hir::stmt::Pattern {
                kind,
                typing: None,
                loc: ctx.loc.clone(),
            })
        }
    }
}

impl AstIntoHir<hir::ctx::Simple<'_>> for ast::stmt::LetDecl<ast::Expr> {
    type Hir = hir::Stmt<hir::Expr>;

    // pre-condition: NOTHING is in symbol table
    // post-condition: construct HIR statement and check identifiers good use
    fn into_hir(self, ctx: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        let loc = Location::default();
        // stmts should be ordered in functions
        // then patterns are stored in order
        self.typed_pattern.store(true, ctx.symbols, ctx.errors)?;
        let expr = self
            .expr
            .into_hir(&mut ctx.add_pat_loc(Some(&self.typed_pattern), &loc))?;
        let pattern = self.typed_pattern.into_hir(&mut ctx.add_loc(&loc))?;

        Ok(hir::Stmt { pattern, expr, loc })
    }
}

mod stream_impl {
    prelude! {
        ast::{ symbol::SymbolKind, stream },
        itertools::Itertools,
    }

    impl AstIntoHir<hir::ctx::PatLoc<'_>> for stream::When {
        type Hir = hir::stream::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc) -> TRes<Self::Hir> {
            // pre-condition: identifiers are stored in symbol table
            // post-condition: construct HIR expression kind and check identifiers good use
            let loc = Location::default();
            let mut arms = vec![];

            // create map from event_id to index in tuple pattern
            let (events_indices, events_nb, no_event_tuple) = {
                let mut events_indices = HashMap::with_capacity(arms.len());
                let mut idx = 0;
                self.pattern.place_events(
                    &mut events_indices,
                    &mut idx,
                    ctx.symbols,
                    ctx.errors,
                )?;
                // default event_pattern tuple
                let no_event_tuple: Vec<_> =
                    std::iter::repeat(hir::pattern::init(hir::pattern::Kind::default()))
                        .take(idx)
                        .collect();
                (events_indices, idx, no_event_tuple)
            };

            // create action arm
            {
                ctx.symbols.local();

                // set local context + create matched pattern
                let (match_pattern, guard) = {
                    let mut elements = no_event_tuple;
                    let opt_rising_edges = self.pattern.create_tuple_pattern(
                        &mut elements,
                        &events_indices,
                        ctx.symbols,
                        ctx.errors,
                    )?;
                    let matched = hir::pattern::init(hir::pattern::Kind::tuple(elements));

                    // transform AST guard into HIR
                    let mut guard = self
                        .guard
                        .map(|expression| expression.into_hir(ctx))
                        .transpose()?;
                    // add rising edge detection to the guard
                    if let Some(rising_edges) = opt_rising_edges {
                        if let Some(old_guard) = guard.take() {
                            guard = Some(hir::stream::expr(hir::stream::Kind::expr(
                                hir::expr::Kind::binop(BOp::And, old_guard, rising_edges),
                            )));
                        } else {
                            guard = Some(rising_edges)
                        }
                    };

                    (matched, guard)
                };
                // transform into HIR
                let expr = self.expr.into_hir(ctx)?;
                ctx.symbols.global();
                arms.push((match_pattern, guard, vec![], expr));
            }

            // create default arm
            {
                let match_pattern = hir::Pattern {
                    kind: hir::pattern::Kind::Default,
                    typing: None,
                    loc: loc.clone(),
                };
                let pat = ctx.pat.expect("there should be a pattern");
                // wraps events in 'none' and signals in 'fby'
                let expr = pat.into_default_expr(&HashMap::new(), ctx.symbols, ctx.errors)?;
                arms.push((match_pattern, None, vec![], expr))
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

    impl AstIntoHir<hir::ctx::PatLoc<'_>> for ast::stream::Expr {
        type Hir = hir::stream::Expr;

        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR stream expression and check identifiers good use
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc) -> TRes<Self::Hir> {
            use hir::stream::Kind;
            let kind = match self {
                stream::Expr::Application(app) => match *app.fun {
                    stream::Expr::Identifier(node) if ctx.symbols.is_node(&node, false) => {
                        let called_node_id =
                            ctx.symbols
                                .get_node_id(&node, false, ctx.loc.clone(), ctx.errors)?;
                        let node_symbol = ctx
                            .symbols
                            .get_symbol(called_node_id)
                            .expect("there should be a symbol")
                            .clone();
                        match node_symbol.kind() {
                            SymbolKind::Node { inputs, .. } => {
                                // check inputs and node_inputs have the same length
                                if inputs.len() != app.inputs.len() {
                                    let error = Error::ArityMismatch {
                                        input_count: app.inputs.len(),
                                        arity: inputs.len(),
                                        loc: ctx.loc.clone(),
                                    };
                                    ctx.errors.push(error);
                                    return Err(TerminationError);
                                }

                                Kind::call(
                                    called_node_id,
                                    app.inputs
                                        .into_iter()
                                        .zip(inputs)
                                        .map(|(input, id)| Ok((*id, input.clone().into_hir(ctx)?)))
                                        .collect::<TRes<Vec<_>>>()?,
                                )
                            }
                            _ => unreachable!(),
                        }
                    }
                    fun => Kind::expr(hir::expr::Kind::app(
                        fun.into_hir(ctx)?,
                        app.inputs
                            .into_iter()
                            .map(|input| input.clone().into_hir(ctx))
                            .collect::<TRes<Vec<_>>>()?,
                    )),
                },
                stream::Expr::Last(last) => {
                    let default = Kind::expr(hir::expr::Kind::constant(Constant::Default));
                    let constant = last
                        .constant
                        .map_or(Ok(hir::stream::expr(default)), |cst| {
                            // check the constant expression is indeed constant
                            cst.check_is_constant(ctx.symbols, ctx.errors)?;
                            cst.into_hir(ctx)
                        })?;

                    let id = ctx.symbols.get_identifier_id(
                        &last.ident.to_string(),
                        false,
                        ctx.loc.clone(),
                        ctx.errors,
                    )?;

                    Kind::FollowedBy {
                        constant: Box::new(constant),
                        id,
                    }
                }
                stream::Expr::Emit(emit) => Kind::some_event(emit.expr.into_hir(ctx)?),
                stream::Expr::Constant(constant) => Kind::Expression {
                    expr: hir::expr::Kind::Constant { constant },
                },
                stream::Expr::Identifier(id) => {
                    let id = ctx
                        .symbols
                        .get_identifier_id(&id, false, ctx.loc.clone(), &mut vec![])
                        .or_else(|_| {
                            ctx.symbols
                                .get_function_id(&id, false, ctx.loc.clone(), ctx.errors)
                        })?;
                    Kind::expr(hir::expr::Kind::Identifier { id })
                }
                stream::Expr::UnOp(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Binop(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::IfThenElse(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::TypedAbstraction(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Structure(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Tuple(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Enumeration(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Array(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Match(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::FieldAccess(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::TupleElementAccess(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Map(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Fold(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Sort(expr) => Kind::expr(expr.into_hir(ctx)?),
                stream::Expr::Zip(expr) => Kind::expr(expr.into_hir(ctx)?),
            };
            Ok(hir::stream::Expr {
                kind,
                typing: None,
                loc: ctx.loc.clone(),
                dependencies: hir::Dependencies::new(),
            })
        }
    }

    impl AstIntoHir<hir::ctx::PatLoc<'_>> for stream::ReactExpr {
        type Hir = hir::stream::Expr;

        // pre-condition: identifiers are stored in symbol table
        // post-condition: construct HIR stream expression and check identifiers good use
        fn into_hir(self, ctx: &mut hir::ctx::PatLoc) -> TRes<Self::Hir> {
            match self {
                stream::ReactExpr::Expr(expr) => expr.into_hir(ctx),
                stream::ReactExpr::When(expr) => {
                    let kind = expr.into_hir(ctx)?;
                    Ok(hir::stream::Expr {
                        kind,
                        typing: None,
                        loc: ctx.loc.clone(),
                        dependencies: hir::Dependencies::new(),
                    })
                }
            }
        }
    }
}

impl AstIntoHir<hir::ctx::Loc<'_>> for Typ {
    type Hir = Typ;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctx: &mut hir::ctx::Loc) -> TRes<Typ> {
        use syn::punctuated::{Pair, Punctuated};
        // pre-condition: Typedefs are stored in symbol table
        // post-condition: construct a new Type without `Typ::NotDefinedYet`
        match self {
                Typ::Array { bracket_token, ty, semi_token, size } => Ok(Typ::Array {
                    bracket_token,
                    ty: Box::new(ty.into_hir(ctx)?),
                    semi_token,
                    size
                }),
                Typ::Tuple { paren_token, elements } => Ok(Typ::Tuple {
                    paren_token,
                    elements: elements
                        .into_pairs()
                        .map(|pair| {
                            let (ty, comma) = pair.into_tuple();
                            let ty = ty.into_hir(ctx)?;
                            Ok(Pair::new(ty, comma))
                        }).collect::<TRes<_>>()?
                }),
                Typ::NotDefinedYet(name) => ctx.symbols
                    .get_struct_id(&name.to_string(), false, ctx.loc.clone(), &mut vec![])
                    .map(|id| Typ::Structure { name: name.clone(), id })
                    .or_else(|_| {
                        ctx.symbols
                            .get_enum_id(&name.to_string(), false, ctx.loc.clone(), &mut vec![])
                            .map(|id| Typ::enumeration(name.clone(), id))
                    }).or_else(|_| {
                        let id = ctx
                            .symbols
                            .get_array_id(&name.to_string(), false, ctx.loc.clone(), ctx.errors)?;
                        Ok(ctx.symbols.get_array(id))
                    }),
                Typ::Abstract { paren_token, inputs, arrow_token, output } => {
                    let inputs = inputs.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.into_hir(ctx)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?;
                    let output = output.into_hir(ctx)?;
                    Ok(Typ::Abstract { paren_token, inputs, arrow_token, output: output.into() })
                }
                Typ::SMEvent { ty, question_token } => Ok(Typ::SMEvent {
                    ty: Box::new(ty.into_hir(ctx)?),
                    question_token
                }),
                Typ::Signal { signal_token, ty } => Ok(Typ::Signal {
                    signal_token,
                    ty: Box::new(ty.into_hir(ctx)?),
                }),
                Typ::Event { event_token, ty } => Ok(Typ::Event {
                    event_token,
                    ty: Box::new(ty.into_hir(ctx)?),
                }),
                Typ::Integer(_) | Typ::Float(_) | Typ::Boolean(_) | Typ::Unit(_) => Ok(self),
                Typ::Enumeration { .. }    // no enumeration at this time: they are `NotDefinedYet`
                | Typ::Structure { .. }    // no structure at this time: they are `NotDefinedYet`
                | Typ::Any                 // users can not write `Any` type
                | Typ::Polymorphism(_)     // users can not write `Polymorphism` type
                 => unreachable!(),
            }
    }
}

impl AstIntoHir<hir::ctx::Simple<'_>> for ast::Typedef {
    type Hir = hir::Typedef;

    // pre-condition: typedefs are already stored in symbol table
    // post-condition: construct HIR typedef and check identifiers good use
    fn into_hir(self, ctx: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        use ast::{Colon, Typedef};
        let loc = Location::default();
        match self {
            Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                let type_id = ctx
                    .symbols
                    .get_struct_id(&id, false, loc.clone(), ctx.errors)?;
                let field_ids = ctx.symbols.get_struct_fields(type_id).clone();

                // insert field's type in symbol table
                field_ids
                    .iter()
                    .zip(fields)
                    .map(
                        |(
                            id,
                            Colon {
                                left: ident,
                                right: typing,
                                ..
                            },
                        )| {
                            let name = ident.to_string();
                            debug_assert_eq!(&name, ctx.symbols.get_name(*id));
                            let typing = typing.into_hir(&mut ctx.add_loc(&loc))?;
                            Ok(ctx.symbols.set_type(*id, typing))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Structure { fields: field_ids },
                    loc,
                })
            }

            Typedef::Enumeration { ident, .. } => {
                let id = ident.to_string();
                let type_id = ctx
                    .symbols
                    .get_enum_id(&id, false, loc.clone(), ctx.errors)?;
                let element_ids = ctx.symbols.get_enum_elements(type_id).clone();
                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Enumeration {
                        elements: element_ids,
                    },
                    loc,
                })
            }

            Typedef::Array {
                ident, array_type, ..
            } => {
                let id = ident.to_string();
                let type_id = ctx
                    .symbols
                    .get_array_id(&id, false, loc.clone(), ctx.errors)?;

                // insert array's type in symbol table
                let typing = array_type.into_hir(&mut ctx.add_loc(&loc))?;
                ctx.symbols.set_array_type(type_id, typing);

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Array,
                    loc,
                })
            }
        }
    }
}
