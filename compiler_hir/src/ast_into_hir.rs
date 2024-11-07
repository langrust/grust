prelude! {
    itertools::Itertools,
}

/// AST transformation into HIR.
pub trait IntoHir<Ctxt> {
    /// Corresponding HIR construct.
    type Hir;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut Ctxt) -> TRes<Self::Hir>;
}

impl IntoHir<hir::ctx::Simple<'_>> for Ast {
    type Hir = hir::File;

    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        // initialize symbol table with builtin operators
        ctxt.syms.initialize();

        // store elements in symbol table
        self.store(ctxt.syms, ctxt.errors)?;

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
                        ast::Item::Component(component) => {
                            components.push(component.into_hir(ctxt))
                        }
                        ast::Item::Function(function) => functions.push(function.into_hir(ctxt)),
                        ast::Item::Typedef(typedef) => typedefs.push(typedef.into_hir(ctxt)),
                        ast::Item::Service(service) => services.push(service.into_hir(ctxt)),
                        ast::Item::Import(import) => imports.push(
                            import
                                .into_hir(ctxt)
                                .map(|res| (ctxt.syms.get_fresh_id(), res)),
                        ),
                        ast::Item::Export(export) => exports.push(
                            export
                                .into_hir(ctxt)
                                .map(|res| (ctxt.syms.get_fresh_id(), res)),
                        ),
                        ast::Item::ComponentImport(component) => {
                            components.push(component.into_hir(ctxt))
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
            location: Location::default(),
        })
    }
}

mod interface_impl {
    prelude! {
        ast::interface::{
            Constrains, FlowDeclaration, FlowExport, FlowImport,
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

    impl<'a> IntoHir<hir::ctx::Simple<'a>> for Service {
        type Hir = hir::Service;

        fn into_hir(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let id = ctxt.syms.insert_service(
                self.ident.to_string(),
                true,
                Location::default(),
                ctxt.errors,
            )?;

            let constrains = if let Some(Constrains { min, max, .. }) = self.constrains {
                (min.base10_parse().unwrap(), max.base10_parse().unwrap())
            } else {
                (10, 500)
            };

            ctxt.syms.local();
            let statements = self
                .flow_statements
                .into_iter()
                .map(|flow_statement| {
                    flow_statement
                        .into_hir(ctxt)
                        .map(|res| (ctxt.syms.get_fresh_id(), res))
                })
                .collect::<TRes<HashMap<_, _>>>()?;
            let graph = Default::default();
            ctxt.syms.global();

            Ok(hir::Service {
                id,
                constrains,
                statements,
                graph,
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Simple<'a>> for FlowImport {
        type Hir = HIRFlowImport;

        fn into_hir(mut self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let location = Location::default();

            let last = self.typed_path.left.segments.pop().unwrap().into_value();
            let name = last.ident.to_string();
            assert!(last.arguments.is_none());
            let path = self.typed_path.left;
            let flow_type = {
                let inner = self
                    .typed_path
                    .right
                    .into_hir(&mut ctxt.add_loc(&location))?;
                match self.kind {
                    FlowKind::Signal(_) => Typ::signal(inner),
                    FlowKind::Event(_) => Typ::event(inner),
                }
            };
            let id = ctxt.syms.insert_flow(
                name,
                Some(path.clone()),
                self.kind,
                flow_type.clone(),
                true,
                location.clone(),
                ctxt.errors,
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

    impl<'a> IntoHir<hir::ctx::Simple<'a>> for FlowExport {
        type Hir = HIRFlowExport;

        fn into_hir(mut self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let location = Location::default();

            let last = self.typed_path.left.segments.pop().unwrap().into_value();
            let name = last.ident.to_string();
            assert!(last.arguments.is_none());
            let path = self.typed_path.left;
            let flow_type = {
                let inner = self
                    .typed_path
                    .right
                    .into_hir(&mut ctxt.add_loc(&location))?;
                match self.kind {
                    FlowKind::Signal(_) => Typ::signal(inner),
                    FlowKind::Event(_) => Typ::event(inner),
                }
            };
            let id = ctxt.syms.insert_flow(
                name,
                Some(path.clone()),
                self.kind,
                flow_type.clone(),
                true,
                location.clone(),
                ctxt.errors,
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

    impl<'a> IntoHir<hir::ctx::Simple<'a>> for FlowStatement {
        type Hir = HIRFlowStatement;

        fn into_hir(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            match self {
                FlowStatement::Declaration(FlowDeclaration {
                    let_token,
                    typed_pattern,
                    eq_token,
                    flow_expression,
                    semi_token,
                }) => {
                    let pattern = typed_pattern.into_hir(ctxt)?;
                    let flow_expression =
                        flow_expression.into_hir(&mut ctxt.add_loc(&Location::default()))?;

                    Ok(HIRFlowStatement::Declaration(HIRFlowDeclaration {
                        let_token,
                        pattern,
                        eq_token,
                        flow_expression,
                        semi_token,
                    }))
                }
                FlowStatement::Instantiation(FlowInstantiation {
                    pattern,
                    eq_token,
                    flow_expression,
                    semi_token,
                }) => {
                    // transform pattern and check its identifiers exist
                    let pattern = pattern.into_hir(ctxt)?;
                    // transform the expression
                    let flow_expression =
                        flow_expression.into_hir(&mut ctxt.add_loc(&Location::default()))?;

                    Ok(HIRFlowStatement::Instantiation(HIRFlowInstantiation {
                        pattern,
                        eq_token,
                        flow_expression,
                        semi_token,
                    }))
                }
            }
        }
    }

    impl<'a> IntoHir<hir::ctx::Simple<'a>> for FlowPattern {
        type Hir = hir::stmt::Pattern;

        fn into_hir(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
            let location = Location::default();

            match self {
                FlowPattern::Single { ident } => {
                    let id = ctxt.syms.get_flow_id(
                        &ident.to_string(),
                        false,
                        location.clone(),
                        ctxt.errors,
                    )?;
                    let typing = ctxt.syms.get_type(id);

                    Ok(hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Identifier { id },
                        typing: Some(typing.clone()),
                        location,
                    })
                }
                FlowPattern::SingleTyped {
                    kind, ident, ty, ..
                } => {
                    let inner = ty.into_hir(&mut ctxt.add_loc(&location))?;
                    let flow_typing = match kind {
                        FlowKind::Signal(_) => Typ::signal(inner),
                        FlowKind::Event(_) => Typ::event(inner),
                    };
                    let id = ctxt.syms.insert_flow(
                        ident.to_string(),
                        None,
                        kind,
                        flow_typing.clone(),
                        true,
                        location.clone(),
                        ctxt.errors,
                    )?;

                    Ok(hir::stmt::Pattern {
                        kind: hir::stmt::Kind::Typed {
                            id,
                            typing: flow_typing.clone(),
                        },
                        typing: Some(flow_typing),
                        location,
                    })
                }
                FlowPattern::Tuple { patterns, .. } => {
                    let elements = patterns
                        .into_iter()
                        .map(|pattern| pattern.into_hir(ctxt))
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
                        location,
                    })
                }
            }
        }
    }
}

impl IntoHir<ctx::Simple<'_>> for ast::Function {
    type Hir = Function;

    // precondition: function and its inputs are already stored in symbol table
    // postcondition: construct HIR function and check identifiers good use
    fn into_hir(self, ctxt: &mut ctx::Simple) -> TRes<Self::Hir> {
        let name = self.ident.to_string();
        let location = Location::default();
        let id = ctxt
            .syms
            .get_function_id(&name, false, location.clone(), ctxt.errors)?;

        // create local context with all signals
        ctxt.syms.local();
        ctxt.syms.restore_context(id);

        // insert function output type in symbol table
        let output_typing = self.output_type.into_hir(&mut ctxt.add_loc(&location))?;
        if !self.contract.clauses.is_empty() {
            let _ = ctxt.syms.insert_function_result(
                output_typing.clone(),
                true,
                location.clone(),
                ctxt.errors,
            )?;
        }
        ctxt.syms.set_function_output_type(id, output_typing);

        let (statements, returned) = self.statements.into_iter().fold(
            (vec![], None),
            |(mut declarations, option_returned), statement| match statement {
                ast::Stmt::Declaration(declaration) => {
                    declarations.push(declaration.into_hir(ctxt));
                    (declarations, option_returned)
                }
                ast::Stmt::Return(ast::stmt::Return { expression, .. }) => {
                    assert!(option_returned.is_none());
                    (
                        declarations,
                        Some(expression.into_hir(&mut ctxt.add_pat_loc(None, &location))),
                    )
                }
            },
        );
        let contract = self.contract.into_hir(ctxt)?;

        ctxt.syms.global();

        Ok(hir::Function {
            id,
            contract,
            statements: statements.into_iter().collect::<TRes<Vec<_>>>()?,
            returned: returned.unwrap()?,
            location,
        })
    }
}

impl IntoHir<ctx::Simple<'_>> for ast::Component {
    type Hir = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn into_hir(self, ctxt: &mut ctx::Simple) -> TRes<Self::Hir> {
        let name = self.ident.to_string();
        let location = Location::default();
        let id = ctxt
            .syms
            .get_node_id(&name, false, location.clone(), ctxt.errors)?;

        // create local context with all signals
        ctxt.syms.local();
        ctxt.syms.restore_context(id);

        let statements = self
            .equations
            .into_iter()
            .map(|equation| equation.into_hir(ctxt))
            .collect::<TRes<Vec<_>>>()?;
        let contract = self.contract.into_hir(ctxt)?;

        ctxt.syms.global();

        Ok(hir::Component::Definition(hir::ComponentDefinition {
            id,
            statements,
            contract,
            location,
            graph: graph::DiGraphMap::new(),
            reduced_graph: graph::DiGraphMap::new(),
            memory: hir::Memory::new(),
        }))
    }
}

impl IntoHir<ctx::Simple<'_>> for ast::ComponentImport {
    type Hir = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node
    fn into_hir(self, ctxt: &mut ctx::Simple) -> TRes<Self::Hir> {
        let last = self.path.clone().segments.pop().unwrap().into_value();
        let name = last.ident.to_string();
        assert!(last.arguments.is_none());

        let location = Location::default();
        let id = ctxt
            .syms
            .get_node_id(&name, false, location.clone(), ctxt.errors)?;

        Ok(hir::Component::Import(hir::ComponentImport {
            id,
            path: self.path,
            location,
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

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for ast::interface::Sample {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Sample {
                flow_expression: Box::new(self.flow_expression.into_hir(ctxt)?),
                period_ms: self.period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Scan {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Scan {
                flow_expression: Box::new(self.flow_expression.into_hir(ctxt)?),
                period_ms: self.period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Timeout {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Timeout {
                flow_expression: Box::new(self.flow_expression.into_hir(ctxt)?),
                deadline: self.deadline.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Throttle {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Throttle {
                flow_expression: Box::new(self.flow_expression.into_hir(ctxt)?),
                delta: self.delta,
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for OnChange {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::OnChange {
                flow_expression: Box::new(self.flow_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Merge {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            Ok(hir::flow::Kind::Merge {
                flow_expression_1: Box::new(self.flow_expression_1.into_hir(ctxt)?),
                flow_expression_2: Box::new(self.flow_expression_2.into_hir(ctxt)?),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for ComponentCall {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let name = self.ident_component.to_string();

            // get called component id
            let component_id =
                ctxt.syms
                    .get_node_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;

            let component_inputs = ctxt.syms.get_node_inputs(component_id).clone();

            // check inputs and node_inputs have the same length
            if self.inputs.len() != component_inputs.len() {
                let error = Error::ArityMismatch {
                    input_count: self.inputs.len(),
                    arity: component_inputs.len(),
                    location: ctxt.loc.clone(),
                };
                ctxt.errors.push(error);
                return Err(TerminationError);
            }

            // transform inputs and map then to the identifiers of the component inputs
            let inputs = self
                .inputs
                .into_iter()
                .zip(component_inputs)
                .map(|(input, id)| Ok((id, input.into_hir(ctxt)?)))
                .collect::<TRes<Vec<_>>>()?;

            Ok(hir::flow::Kind::ComponentCall {
                component_id,
                inputs,
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for FlowExpression {
        type Hir = flow::Expr;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression and check identifiers good use
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<Self::Hir> {
            let kind = match self {
                FlowExpression::Ident(ident) => {
                    let id = ctxt
                        .syms
                        .get_flow_id(&ident, false, ctxt.loc.clone(), ctxt.errors)?;
                    flow::Kind::Ident { id }
                }
                FlowExpression::ComponentCall(flow_expression) => flow_expression.into_hir(ctxt)?,
                FlowExpression::Sample(flow_expression) => flow_expression.into_hir(ctxt)?,
                FlowExpression::Scan(flow_expression) => flow_expression.into_hir(ctxt)?,
                FlowExpression::Timeout(flow_expression) => flow_expression.into_hir(ctxt)?,
                FlowExpression::Throttle(flow_expression) => flow_expression.into_hir(ctxt)?,
                FlowExpression::OnChange(flow_expression) => flow_expression.into_hir(ctxt)?,
                FlowExpression::Merge(flow_expression) => flow_expression.into_hir(ctxt)?,
            };
            Ok(flow::Expr {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
            })
        }
    }
}

impl IntoHir<ctx::Simple<'_>> for ast::Contract {
    type Hir = Contract;

    fn into_hir(self, ctxt: &mut ctx::Simple) -> TRes<Self::Hir> {
        use ast::contract::ClauseKind;
        let (requires, ensures, invariant) = self.clauses.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut requires, mut ensures, mut invariant), clause| {
                match clause.kind {
                    ClauseKind::Requires(_) => requires.push(clause.term.into_hir(ctxt)),
                    ClauseKind::Ensures(_) => ensures.push(clause.term.into_hir(ctxt)),
                    ClauseKind::Invariant(_) => invariant.push(clause.term.into_hir(ctxt)),
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

impl<'a> IntoHir<ctx::Simple<'a>> for ast::contract::Term {
    type Hir = hir::contract::Term;

    fn into_hir(self, ctxt: &mut ctx::Simple<'a>) -> TRes<Self::Hir> {
        use ast::contract::*;
        let location = Location::default();
        match self {
            Term::Result(_) => {
                let id = ctxt
                    .syms
                    .get_function_result_id(false, location.clone(), ctxt.errors)?;
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::ident(id),
                    None,
                    location,
                ))
            }
            Term::Implication(Implication { left, right, .. }) => {
                let left = left.into_hir(ctxt)?;
                let right = right.into_hir(ctxt)?;

                Ok(hir::contract::Term::new(
                    hir::contract::Kind::implication(left, right),
                    None,
                    location,
                ))
            }
            Term::Enumeration(Enumeration {
                enum_name,
                elem_name,
            }) => {
                let enum_id =
                    ctxt.syms
                        .get_enum_id(&enum_name, false, location.clone(), ctxt.errors)?;
                let element_id = ctxt.syms.get_enum_elem_id(
                    &elem_name,
                    &enum_name,
                    false,
                    location.clone(),
                    ctxt.errors,
                )?;
                // TODO check elem is in enum
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::enumeration(enum_id, element_id),
                    None,
                    location,
                ))
            }
            Term::Unary(Unary { op, term }) => Ok(hir::contract::Term::new(
                hir::contract::Kind::unary(op, term.into_hir(ctxt)?),
                None,
                location,
            )),
            Term::Binary(Binary { op, left, right }) => Ok(hir::contract::Term::new(
                hir::contract::Kind::binary(op, left.into_hir(ctxt)?, right.into_hir(ctxt)?),
                None,
                location,
            )),
            Term::Constant(constant) => Ok(hir::contract::Term::new(
                hir::contract::Kind::constant(constant),
                None,
                location,
            )),
            Term::Identifier(ident) => {
                let id =
                    ctxt.syms
                        .get_identifier_id(&ident, false, location.clone(), ctxt.errors)?;
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::ident(id),
                    None,
                    location,
                ))
            }
            Term::ForAll(ForAll {
                ident, ty, term, ..
            }) => {
                let ty = ty.into_hir(&mut ctxt.add_loc(&location))?;
                ctxt.syms.local();
                let id = ctxt.syms.insert_identifier(
                    ident.clone(),
                    Some(ty),
                    true,
                    location.clone(),
                    ctxt.errors,
                )?;
                let term = term.into_hir(ctxt)?;
                ctxt.syms.global();
                Ok(hir::contract::Term::new(
                    hir::contract::Kind::forall(id, term),
                    None,
                    location,
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
                    ctxt.syms
                        .get_identifier_id(&event, false, location.clone(), ctxt.errors)?;
                ctxt.syms.local();
                // set pattern signal in local context
                let pattern_id = ctxt.syms.insert_identifier(
                    pattern.clone(),
                    None,
                    true,
                    location.clone(),
                    ctxt.errors,
                )?;
                // transform term into HIR
                let right = term.into_hir(ctxt)?;
                ctxt.syms.global();
                // construct right side of implication: `PresentEvent(pat) == event`
                let left = hir::contract::Term::new(
                    hir::contract::Kind::binary(
                        BOp::Eq,
                        hir::contract::Term::new(
                            hir::contract::Kind::present(event_id, pattern_id),
                            None,
                            location.clone(),
                        ),
                        hir::contract::Term::new(
                            hir::contract::Kind::ident(event_id),
                            None,
                            location.clone(),
                        ),
                    ),
                    None,
                    location.clone(),
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
                            location.clone(),
                        ),
                    ),
                    None,
                    location,
                );
                Ok(term)
            }
        }
    }
}

impl IntoHir<ctx::Simple<'_>> for ast::Eq {
    type Hir = hir::stream::Stmt;

    /// Pre-condition: equation's signal is already stored in symbol table.
    ///
    /// Post-condition: construct HIR equation and check identifiers good use.
    fn into_hir(self, ctxt: &mut ctx::Simple) -> TRes<Self::Hir> {
        use ast::{
            equation::{Arm, Eq, Instantiation, Match},
            stmt::LetDecl,
        };
        let location = Location::default();

        // get signals defined by the equation
        let mut defined_signals = HashMap::new();
        self.get_signals(&mut defined_signals, ctxt.syms, ctxt.errors)?;

        match self {
            Eq::LocalDef(LetDecl {
                expression,
                typed_pattern: pattern,
                ..
            })
            | Eq::OutputDef(Instantiation {
                expression,
                pattern,
                ..
            }) => {
                let expression =
                    expression.into_hir(&mut ctxt.add_pat_loc(Some(&pattern), &location))?;
                let pattern = pattern.into_hir(&mut ctxt.add_loc(&location))?;
                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
            Eq::Match(Match {
                expression, arms, ..
            }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().into_hir(&mut ctxt.add_loc(&location)))
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
                                ctxt.syms.local();

                                // set local context: pattern signals + equations' signals
                                pattern.store(ctxt.syms, ctxt.errors)?;
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctxt.syms,
                                            ctxt.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform pattern guard and equations into HIR with local context
                                let pattern = pattern.into_hir(&mut ctxt.add_loc(&location))?;
                                let guard = guard
                                    .map(|(_, expression)| {
                                        expression.into_hir(&mut ctxt.add_pat_loc(None, &location))
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| equation.into_hir(ctxt))
                                    .collect::<TRes<Vec<_>>>()?;

                                ctxt.syms.global();

                                (signals, pattern, guard, statements)
                            };

                            // create the tuple expression
                            let expression = {
                                // check defined signals are all the same
                                if defined_signals.len() != signals.len() {
                                    let error = Error::IncompatibleMatchStatements {
                                        expected: defined_signals.len(),
                                        received: signals.len(),
                                        location: location.clone(),
                                    };
                                    ctxt.errors.push(error);
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
                                                location: location.clone(),
                                            };
                                            ctxt.errors.push(error);
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
                let expression = stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                    expression.into_hir(&mut ctxt.add_pat_loc(None, &location))?,
                    arms,
                )));

                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
        }
    }
}

impl IntoHir<ctx::Simple<'_>> for ast::ReactEq {
    type Hir = stream::Stmt;

    /// Pre-condition: equation's signal is already stored in symbol table.
    ///
    /// Post-condition: construct HIR equation and check identifiers good use.
    fn into_hir(self, ctxt: &mut ctx::Simple) -> TRes<Self::Hir> {
        use ast::{
            equation::{Arm, EventArmWhen, Instantiation, MatchWhen},
            stmt::LetDecl,
            ReactEq,
        };
        let location = Location::default();

        // get signals defined by the equation
        let mut defined_signals = HashMap::new();
        self.get_signals(&mut defined_signals, ctxt.syms, ctxt.errors)?;

        match self {
            ReactEq::LocalDef(LetDecl {
                expression,
                typed_pattern: pattern,
                ..
            })
            | ReactEq::OutputDef(Instantiation {
                expression,
                pattern,
                ..
            }) => {
                let expression =
                    expression.into_hir(&mut ctxt.add_pat_loc(Some(&pattern), &location))?;
                let pattern = pattern.into_hir(&mut ctxt.add_loc(&location))?;
                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
            }
            ReactEq::Match(ast::equation::Match {
                expression, arms, ..
            }) => {
                // create the receiving pattern for the equation
                let pattern = {
                    let mut elements = defined_signals
                        .values()
                        .map(|pat| pat.clone().into_hir(&mut ctxt.add_loc(&location)))
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
                                ctxt.syms.local();

                                // set local context: pattern signals + equations' signals
                                pattern.store(ctxt.syms, ctxt.errors)?;
                                let mut signals = HashMap::new();
                                equations
                                    .iter()
                                    .map(|equation| {
                                        // store equations' signals in the local context
                                        equation.store_signals(
                                            true,
                                            &mut signals,
                                            ctxt.syms,
                                            ctxt.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;

                                // transform pattern guard and equations into HIR with local context
                                let pattern = pattern.into_hir(&mut ctxt.add_loc(&location))?;
                                let guard = guard
                                    .map(|(_, expression)| {
                                        expression.into_hir(&mut ctxt.add_pat_loc(None, &location))
                                    })
                                    .transpose()?;
                                let statements = equations
                                    .into_iter()
                                    .map(|equation| equation.into_hir(ctxt))
                                    .collect::<TRes<Vec<_>>>()?;

                                ctxt.syms.global();

                                (signals, pattern, guard, statements)
                            };

                            // create the tuple expression
                            let expression = {
                                // check defined signals are all the same
                                if defined_signals.len() != signals.len() {
                                    let error = Error::IncompatibleMatchStatements {
                                        expected: defined_signals.len(),
                                        received: signals.len(),
                                        location: location.clone(),
                                    };
                                    ctxt.errors.push(error);
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
                                                location: location.clone(),
                                            };
                                            ctxt.errors.push(error);
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
                let expression = stream::expr(stream::Kind::expr(hir::expr::Kind::match_expr(
                    expression.into_hir(&mut ctxt.add_pat_loc(None, &location))?,
                    arms,
                )));

                Ok(hir::Stmt {
                    pattern,
                    expression,
                    location,
                })
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
                                ctxt.syms,
                                ctxt.errors,
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
                        ctxt.syms,
                        ctxt.errors,
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
                            ctxt.syms.local();

                            // set local context + create matched pattern
                            let (matched_pattern, guard) = {
                                let mut elements = default_pattern.clone();
                                let opt_rising_edges = event_pattern.create_tuple_pattern(
                                    &mut elements,
                                    &events_indices,
                                    ctxt.syms,
                                    ctxt.errors,
                                )?;
                                let matched = pattern::init(pattern::Kind::tuple(elements));

                                // transform AST guard into HIR
                                let mut guard = guard
                                    .map(|(_, expression)| {
                                        expression.into_hir(&mut ctxt.add_pat_loc(None, &location))
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
                                            ctxt.syms,
                                            ctxt.errors,
                                        )
                                    })
                                    .collect::<TRes<()>>()?;
                                signals
                            };

                            // transform equations into HIR with local context
                            let statements = equations
                                .into_iter()
                                .map(|equation| equation.into_hir(ctxt))
                                .collect::<TRes<Vec<_>>>()?;

                            ctxt.syms.global();

                            // create the tuple expression
                            let expression = defined_pattern.into_default_expr(
                                &signals,
                                ctxt.syms,
                                ctxt.errors,
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

                let pattern = defined_pattern.into_hir(&mut ctxt.add_loc(&location))?;

                Ok(hir::Stmt {
                    pattern,
                    expression: match_expr,
                    location,
                })
            }
        }
    }
}

impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Unop<E>
where
    E: IntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Unop {
            op: self.op,
            expression: Box::new(self.expression.into_hir(ctxt)?),
        })
    }
}

impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Binop<E>
where
    E: IntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        let ast::expr::Binop {
            op,
            left_expression,
            right_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Binop {
            op,
            left_expression: Box::new(left_expression.into_hir(ctxt)?),
            right_expression: Box::new(right_expression.into_hir(ctxt)?),
        })
    }
}

impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for ast::expr::IfThenElse<E>
where
    E: IntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        let ast::expr::IfThenElse {
            expression,
            true_expression,
            false_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::IfThenElse {
            expression: Box::new(expression.into_hir(ctxt)?),
            true_expression: Box::new(true_expression.into_hir(ctxt)?),

            false_expression: Box::new(false_expression.into_hir(ctxt)?),
        })
    }
}

impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Application<E>
where
    E: IntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        let ast::expr::Application {
            function_expression,
            inputs,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Application {
            function_expression: Box::new(function_expression.into_hir(ctxt)?),
            inputs: inputs
                .into_iter()
                .map(|input| input.into_hir(ctxt))
                .collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for ast::expr::Structure<E>
where
    E: IntoHir<hir::ctx::PatLoc<'a>>,
{
    type Hir = expr::Kind<E::Hir>;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
        let ast::expr::Structure { name, fields } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        let id = ctxt
            .syms
            .get_struct_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;
        let mut field_ids = ctxt
            .syms
            .get_struct_fields(id)
            .clone()
            .into_iter()
            .map(|id| (ctxt.syms.get_name(id).clone(), id))
            .collect::<HashMap<_, _>>();

        let fields = fields
            .into_iter()
            .map(|(field_name, expression)| {
                let id = field_ids.remove(&field_name).map_or_else(
                    || {
                        let error = Error::UnknownField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: ctxt.loc.clone(),
                        };
                        ctxt.errors.push(error);
                        Err(TerminationError)
                    },
                    |id| Ok(id),
                )?;
                let expression = expression.into_hir(ctxt)?;
                Ok((id, expression))
            })
            .collect::<TRes<Vec<_>>>()?;

        // check if there are no missing fields
        field_ids
            .keys()
            .map(|field_name| {
                let error = Error::MissingField {
                    structure_name: name.clone(),
                    field_name: field_name.clone(),
                    location: ctxt.loc.clone(),
                };
                ctxt.errors.push(error);
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

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Enumeration<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>>
        where
            E: IntoHir<hir::ctx::PatLoc<'a>>,
        {
            let Enumeration {
                enum_name,
                elem_name,
                ..
            } = self;

            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            let enum_id =
                ctxt.syms
                    .get_enum_id(&enum_name, false, ctxt.loc.clone(), ctxt.errors)?;
            let elem_id = ctxt.syms.get_enum_elem_id(
                &elem_name,
                &enum_name,
                false,
                ctxt.loc.clone(),
                ctxt.errors,
            )?;
            // TODO check elem is in enum
            Ok(expr::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Array<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Array { elements } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Array {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Tuple<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Tuple { elements } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Match<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Match { expression, arms } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Match {
                expression: Box::new(expression.into_hir(ctxt)?),
                arms: arms
                    .into_iter()
                    .map(
                        |Arm {
                             pattern,
                             guard,
                             expression,
                         }| {
                            ctxt.syms.local();
                            pattern.store(ctxt.syms, ctxt.errors)?;
                            let pattern = pattern.into_hir(&mut ctxt.remove_pat())?;
                            let guard = guard
                                .map(|expression| expression.into_hir(ctxt))
                                .transpose()?;
                            let expression = expression.into_hir(ctxt)?;
                            ctxt.syms.global();
                            Ok((pattern, guard, vec![], expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for FieldAccess<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let FieldAccess { expression, field } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::FieldAccess {
                expression: Box::new(expression.into_hir(ctxt)?),
                field,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for TupleElementAccess<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let TupleElementAccess {
                expression,
                element_number,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::TupleElementAccess {
                expression: Box::new(expression.into_hir(ctxt)?),
                element_number,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Map<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Map {
                expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Map {
                expression: Box::new(expression.into_hir(ctxt)?),
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Fold<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Fold {
                expression,
                initialization_expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Fold {
                expression: Box::new(expression.into_hir(ctxt)?),
                initialization_expression: Box::new(initialization_expression.into_hir(ctxt)?),
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Sort<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Sort {
                expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Sort {
                expression: Box::new(expression.into_hir(ctxt)?),
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Zip<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Zip { arrays } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|array| array.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for TypedAbstraction<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let TypedAbstraction { inputs, expression } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use

            ctxt.syms.local();
            let inputs = inputs
                .into_iter()
                .map(|(input_name, typing)| {
                    let typing = typing.into_hir(&mut ctxt.remove_pat())?;
                    ctxt.syms.insert_identifier(
                        input_name,
                        Some(typing),
                        true,
                        ctxt.loc.clone(),
                        ctxt.errors,
                    )
                })
                .collect::<TRes<Vec<_>>>()?;
            let expression = expression.into_hir(ctxt)?;
            ctxt.syms.global();

            Ok(expr::Kind::Abstraction {
                inputs,
                expression: Box::new(expression),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::PatLoc<'a>> for ast::Expr {
        type Hir = hir::Expr;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression and check identifiers good use
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<Self::Hir> {
            let kind = match self {
                Self::Constant(constant) => hir::expr::Kind::Constant { constant },
                Self::Identifier(id) => {
                    let id = ctxt
                        .syms
                        .get_identifier_id(&id, false, ctxt.loc.clone(), &mut vec![])
                        .or_else(|_| {
                            ctxt.syms
                                .get_function_id(&id, false, ctxt.loc.clone(), ctxt.errors)
                        })?;
                    hir::expr::Kind::Identifier { id }
                }
                Self::Unop(expression) => expression.into_hir(ctxt)?,
                Self::Binop(expression) => expression.into_hir(ctxt)?,
                Self::IfThenElse(expression) => expression.into_hir(ctxt)?,
                Self::Application(expression) => expression.into_hir(ctxt)?,
                Self::TypedAbstraction(expression) => expression.into_hir(ctxt)?,
                Self::Structure(expression) => expression.into_hir(ctxt)?,
                Self::Tuple(expression) => expression.into_hir(ctxt)?,
                Self::Enumeration(expression) => expression.into_hir(ctxt)?,
                Self::Array(expression) => expression.into_hir(ctxt)?,
                Self::Match(expression) => expression.into_hir(ctxt)?,
                Self::FieldAccess(expression) => expression.into_hir(ctxt)?,
                Self::TupleElementAccess(expression) => expression.into_hir(ctxt)?,
                Self::Map(expression) => expression.into_hir(ctxt)?,
                Self::Fold(expression) => expression.into_hir(ctxt)?,
                Self::Sort(expression) => expression.into_hir(ctxt)?,
                Self::Zip(expression) => expression.into_hir(ctxt)?,
            };
            Ok(hir::Expr {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
                dependencies: Dependencies::new(),
            })
        }
    }
}

mod expr_pattern_impl {
    prelude! {
        ast::expr::{PatEnumeration, PatStructure, PatTuple},
    }

    impl IntoHir<hir::ctx::Loc<'_>> for PatStructure {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let PatStructure { name, fields, rest } = self;

            let id = ctxt
                .syms
                .get_struct_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;
            let mut field_ids = ctxt
                .syms
                .get_struct_fields(id)
                .clone()
                .into_iter()
                .map(|id| (ctxt.syms.get_name(id).clone(), id))
                .collect::<HashMap<_, _>>();

            let fields = fields
                .into_iter()
                .map(|(field_name, optional_pattern)| {
                    let id = field_ids.remove(&field_name).map_or_else(
                        || {
                            let error = Error::UnknownField {
                                structure_name: name.clone(),
                                field_name: field_name.clone(),
                                location: ctxt.loc.clone(),
                            };
                            ctxt.errors.push(error);
                            Err(TerminationError)
                        },
                        |id| Ok(id),
                    )?;
                    let pattern = optional_pattern
                        .map(|pattern| pattern.into_hir(ctxt))
                        .transpose()?;
                    Ok((id, pattern))
                })
                .collect::<TRes<Vec<_>>>()?;

            if rest.is_none() {
                // check if there are no missing fields
                field_ids
                    .keys()
                    .map(|field_name| {
                        let error = Error::MissingField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: ctxt.loc.clone(),
                        };
                        ctxt.errors.push(error);
                        TRes::<()>::Err(TerminationError)
                    })
                    .collect::<TRes<Vec<_>>>()?;
            }

            Ok(hir::pattern::Kind::Structure { id, fields })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for PatEnumeration {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let PatEnumeration {
                enum_name,
                elem_name,
            } = self;

            let enum_id =
                ctxt.syms
                    .get_enum_id(&enum_name, false, ctxt.loc.clone(), ctxt.errors)?;
            let elem_id = ctxt.syms.get_enum_elem_id(
                &elem_name,
                &enum_name,
                false,
                ctxt.loc.clone(),
                ctxt.errors,
            )?;
            Ok(hir::pattern::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for PatTuple {
        type Hir = hir::pattern::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::pattern::Kind> {
            let PatTuple { elements } = self;
            Ok(hir::pattern::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|pattern| pattern.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for ast::expr::Pattern {
        type Hir = hir::Pattern;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<Self::Hir> {
            let kind = match self {
                ast::expr::Pattern::Constant(constant) => hir::pattern::Kind::Constant { constant },
                ast::expr::Pattern::Identifier(name) => {
                    let id =
                        ctxt.syms
                            .get_identifier_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;
                    hir::pattern::Kind::Identifier { id }
                }
                ast::expr::Pattern::Structure(pattern) => pattern.into_hir(ctxt)?,
                ast::expr::Pattern::Enumeration(pattern) => pattern.into_hir(ctxt)?,
                ast::expr::Pattern::Tuple(pattern) => pattern.into_hir(ctxt)?,
                // Pattern::None => hir::pattern::Kind::None,
                ast::expr::Pattern::Default => hir::pattern::Kind::Default,
            };

            Ok(hir::Pattern {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
            })
        }
    }
}

mod stmt_pattern_impl {
    prelude! {
        ast::stmt::{Typed, Tuple},
    }

    impl IntoHir<hir::ctx::Loc<'_>> for Typed {
        type Hir = hir::stmt::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::stmt::Kind> {
            let Typed { ident, typing, .. } = self;

            let id = ctxt.syms.get_identifier_id(
                &ident.to_string(),
                false,
                ctxt.loc.clone(),
                ctxt.errors,
            )?;
            let typing = typing.into_hir(ctxt)?;
            Ok(hir::stmt::Kind::Typed { id, typing })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for Tuple {
        type Hir = hir::stmt::Kind;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<hir::stmt::Kind> {
            let Tuple { elements } = self;
            Ok(hir::stmt::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|pattern| pattern.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl IntoHir<hir::ctx::Loc<'_>> for ast::stmt::Pattern {
        type Hir = hir::stmt::Pattern;

        fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<Self::Hir> {
            let kind = match self {
                ast::stmt::Pattern::Identifier(ident) => {
                    let id = ctxt.syms.get_identifier_id(
                        &ident.to_string(),
                        false,
                        ctxt.loc.clone(),
                        ctxt.errors,
                    )?;
                    hir::stmt::Kind::Identifier { id }
                }
                ast::stmt::Pattern::Typed(pattern) => pattern.into_hir(ctxt)?,
                ast::stmt::Pattern::Tuple(pattern) => pattern.into_hir(ctxt)?,
            };

            Ok(hir::stmt::Pattern {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
            })
        }
    }
}

impl IntoHir<hir::ctx::Simple<'_>> for ast::stmt::LetDecl<ast::Expr> {
    type Hir = hir::Stmt<hir::Expr>;

    // precondition: NOTHING is in symbol table
    // postcondition: construct HIR statement and check identifiers good use
    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        let ast::stmt::LetDecl {
            typed_pattern,
            expression,
            ..
        } = self;
        let location = Location::default();

        // stmts should be ordered in functions
        // then patterns are stored in order
        typed_pattern.store(true, ctxt.syms, ctxt.errors)?;
        let expression =
            expression.into_hir(&mut ctxt.add_pat_loc(Some(&typed_pattern), &location))?;
        let pattern = typed_pattern.into_hir(&mut ctxt.add_loc(&location))?;

        Ok(hir::Stmt {
            pattern,
            expression,
            location,
        })
    }
}

mod stream_impl {
    prelude! {
        ast::{ expr::Application, symbol::SymbolKind, stream },
        itertools::Itertools,
    }

    impl IntoHir<hir::ctx::PatLoc<'_>> for stream::When {
        type Hir = hir::stream::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc) -> TRes<Self::Hir> {
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
                event_pattern.place_events(
                    &mut events_indices,
                    &mut idx,
                    ctxt.syms,
                    ctxt.errors,
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
                        .map(|expression| expression.into_hir(ctxt))
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
                let expression = expression.into_hir(ctxt)?;
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

    impl IntoHir<hir::ctx::PatLoc<'_>> for ast::stream::Expr {
        type Hir = hir::stream::Expr;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR stream expression and check identifiers good use
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc) -> TRes<Self::Hir> {
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
                                    let error = Error::ArityMismatch {
                                        input_count: inputs_stream_expressions.len(),
                                        arity: inputs.len(),
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
                                        .map(|(input, id)| Ok((*id, input.clone().into_hir(ctxt)?)))
                                        .collect::<TRes<Vec<_>>>()?,
                                )
                            }
                            _ => unreachable!(),
                        }
                    }
                    function_expression => hir::stream::Kind::Expression {
                        expression: hir::expr::Kind::Application {
                            function_expression: Box::new(function_expression.into_hir(ctxt)?),
                            inputs: inputs_stream_expressions
                                .into_iter()
                                .map(|input| input.clone().into_hir(ctxt))
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
                        cst.into_hir(ctxt)
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
                    hir::stream::Kind::some_event(expr.into_hir(ctxt)?)
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
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Binop(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::IfThenElse(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::TypedAbstraction(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Structure(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Tuple(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Enumeration(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Array(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Match(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::FieldAccess(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::TupleElementAccess(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Map(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Fold(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Sort(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
                },
                stream::Expr::Zip(expression) => hir::stream::Kind::Expression {
                    expression: expression.into_hir(ctxt)?,
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

    impl IntoHir<hir::ctx::PatLoc<'_>> for stream::ReactExpr {
        type Hir = hir::stream::Expr;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR stream expression and check identifiers good use
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc) -> TRes<Self::Hir> {
            match self {
                stream::ReactExpr::Expr(expr) => expr.into_hir(ctxt),
                stream::ReactExpr::When(expr) => {
                    let kind = expr.into_hir(ctxt)?;
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
}

impl IntoHir<hir::ctx::Loc<'_>> for Typ {
    type Hir = Typ;

    /// Transforms AST into HIR and check identifiers good use.
    fn into_hir(self, ctxt: &mut hir::ctx::Loc) -> TRes<Typ> {
        use syn::punctuated::{Pair, Punctuated};
        // precondition: Typedefs are stored in symbol table
        // postcondition: construct a new Type without `Typ::NotDefinedYet`
        match self {
                Typ::Array { bracket_token, ty, semi_token, size } => Ok(Typ::Array {
                    bracket_token,
                    ty: Box::new(ty.into_hir(ctxt)?),
                    semi_token,
                    size
                }),
                Typ::Tuple { paren_token, elements } => Ok(Typ::Tuple {
                    paren_token,
                    elements: elements.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.into_hir(ctxt)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?
                }),
                Typ::NotDefinedYet(name) => ctxt.syms
                    .get_struct_id(&name.to_string(), false, ctxt.loc.clone(), &mut vec![])
                    .map(|id| Typ::Structure { name: name.clone(), id })
                    .or_else(|_| {
                        ctxt.syms
                            .get_enum_id(&name.to_string(), false, ctxt.loc.clone(), &mut vec![])
                            .map(|id| Typ::Enumeration { name: name.clone(), id })
                    })
                    .or_else(|_| {
                        let id = ctxt.syms.get_array_id(&name.to_string(), false, ctxt.loc.clone(), ctxt.errors)?;
                        Ok(ctxt.syms.get_array(id))
                    }),
                Typ::Abstract { paren_token, inputs, arrow_token, output } => {
                    let inputs = inputs.into_pairs()
                    .map(|pair| {
                        let (ty, comma) = pair.into_tuple();
                        let ty = ty.into_hir(ctxt)?;
                        Ok(Pair::new(ty, comma))
                    }).collect::<TRes<Punctuated<Typ, Token![,]>>>()?;
                    let output = output.into_hir(ctxt)?;
                    Ok(Typ::Abstract { paren_token, inputs, arrow_token, output: output.into() })
                }
                Typ::SMEvent { ty, question_token } => Ok(Typ::SMEvent {
                    ty: Box::new(ty.into_hir(ctxt)?),
                    question_token
                }),
                Typ::Signal { signal_token, ty } => Ok(Typ::Signal {
                    signal_token,
                    ty: Box::new(ty.into_hir(ctxt)?),
                }),
                Typ::Event { event_token, ty } => Ok(Typ::Event {
                    event_token,
                    ty: Box::new(ty.into_hir(ctxt)?),
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

impl IntoHir<hir::ctx::Simple<'_>> for ast::Typedef {
    type Hir = hir::Typedef;

    // precondition: typedefs are already stored in symbol table
    // postcondition: construct HIR typedef and check identifiers good use
    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        use ast::{Colon, Typedef};
        let location = Location::default();
        match self {
            Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                let type_id = ctxt
                    .syms
                    .get_struct_id(&id, false, location.clone(), ctxt.errors)?;
                let field_ids = ctxt.syms.get_struct_fields(type_id).clone();

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
                            debug_assert_eq!(&name, ctxt.syms.get_name(*id));
                            let typing = typing.into_hir(&mut ctxt.add_loc(&location))?;
                            Ok(ctxt.syms.set_type(*id, typing))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?;

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Structure { fields: field_ids },
                    location,
                })
            }

            Typedef::Enumeration { ident, .. } => {
                let id = ident.to_string();
                let type_id = ctxt
                    .syms
                    .get_enum_id(&id, false, location.clone(), ctxt.errors)?;
                let element_ids = ctxt.syms.get_enum_elements(type_id).clone();
                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Enumeration {
                        elements: element_ids,
                    },
                    location,
                })
            }

            Typedef::Array {
                ident, array_type, ..
            } => {
                let id = ident.to_string();
                let type_id = ctxt
                    .syms
                    .get_array_id(&id, false, location.clone(), ctxt.errors)?;

                // insert array's type in symbol table
                let typing = array_type.into_hir(&mut ctxt.add_loc(&location))?;
                ctxt.syms.set_array_type(type_id, typing);

                Ok(hir::Typedef {
                    id: type_id,
                    kind: hir::typedef::Kind::Array,
                    location,
                })
            }
        }
    }
}
