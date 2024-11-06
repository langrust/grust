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
        let Service {
            ident,
            constrains,
            flow_statements,
            ..
        } = self;

        let id =
            ctxt.syms
                .insert_service(ident.to_string(), true, Location::default(), ctxt.errors)?;

        let constrains = if let Some(Constrains { min, max, .. }) = constrains {
            (min.base10_parse().unwrap(), max.base10_parse().unwrap())
        } else {
            (10, 500)
        };

        ctxt.syms.local();
        let statements = flow_statements
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

    fn into_hir(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
        let FlowImport {
            import_token,
            kind,
            mut typed_path,
            semi_token,
        } = self;
        let location = Location::default();

        let last = typed_path.left.segments.pop().unwrap().into_value();
        let name = last.ident.to_string();
        assert!(last.arguments.is_none());
        let path = typed_path.left;
        let flow_type = {
            let inner = typed_path.right.into_hir(&mut ctxt.add_loc(&location))?;
            match kind {
                FlowKind::Signal(_) => Typ::signal(inner),
                FlowKind::Event(_) => Typ::event(inner),
            }
        };
        let id = ctxt.syms.insert_flow(
            name,
            Some(path.clone()),
            kind,
            flow_type.clone(),
            true,
            location.clone(),
            ctxt.errors,
        )?;

        Ok(HIRFlowImport {
            import_token,
            id,
            path,
            colon_token: typed_path.colon,
            flow_type,
            semi_token,
        })
    }
}

impl<'a> IntoHir<hir::ctx::Simple<'a>> for FlowExport {
    type Hir = HIRFlowExport;

    fn into_hir(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::Hir> {
        let FlowExport {
            export_token,
            kind,
            mut typed_path,
            semi_token,
        } = self;
        let location = Location::default();

        let last = typed_path.left.segments.pop().unwrap().into_value();
        let name = last.ident.to_string();
        assert!(last.arguments.is_none());
        let path = typed_path.left;
        let flow_type = {
            let inner = typed_path.right.into_hir(&mut ctxt.add_loc(&location))?;
            match kind {
                FlowKind::Signal(_) => Typ::signal(inner),
                FlowKind::Event(_) => Typ::event(inner),
            }
        };
        let id = ctxt.syms.insert_flow(
            name,
            Some(path.clone()),
            kind,
            flow_type.clone(),
            true,
            location.clone(),
            ctxt.errors,
        )?;

        Ok(HIRFlowExport {
            export_token,
            id,
            path,
            colon_token: typed_path.colon,
            flow_type,
            semi_token,
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

mod flow_expr {
    prelude! {
        ast::interface::{
            FlowExpression, ComponentCall, OnChange, Merge,
            Sample, Scan, Throttle, Timeout,
        },
        hir::flow,
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for ast::interface::Sample {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let Sample {
                flow_expression,
                period_ms,
                ..
            } = self;
            Ok(hir::flow::Kind::Sample {
                flow_expression: Box::new(flow_expression.into_hir(ctxt)?),
                period_ms: period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Scan {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let Scan {
                flow_expression,
                period_ms,
                ..
            } = self;
            Ok(hir::flow::Kind::Scan {
                flow_expression: Box::new(flow_expression.into_hir(ctxt)?),
                period_ms: period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Timeout {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let Timeout {
                flow_expression,
                deadline,
                ..
            } = self;
            Ok(hir::flow::Kind::Timeout {
                flow_expression: Box::new(flow_expression.into_hir(ctxt)?),
                deadline: deadline.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Throttle {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let Throttle {
                flow_expression,
                delta,
                ..
            } = self;
            Ok(hir::flow::Kind::Throttle {
                flow_expression: Box::new(flow_expression.into_hir(ctxt)?),
                delta,
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for OnChange {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let OnChange {
                flow_expression, ..
            } = self;
            Ok(hir::flow::Kind::OnChange {
                flow_expression: Box::new(flow_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for Merge {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let Merge {
                flow_expression_1,
                flow_expression_2,
                ..
            } = self;
            Ok(hir::flow::Kind::Merge {
                flow_expression_1: Box::new(flow_expression_1.into_hir(ctxt)?),
                flow_expression_2: Box::new(flow_expression_2.into_hir(ctxt)?),
            })
        }
    }

    impl<'a> IntoHir<hir::ctx::Loc<'a>> for ComponentCall {
        type Hir = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::Loc<'a>) -> TRes<hir::flow::Kind> {
            let ComponentCall {
                ident_component,
                inputs,
                ..
            } = self;

            let name = ident_component.to_string();

            // get called component id
            let component_id =
                ctxt.syms
                    .get_node_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;

            let component_inputs = ctxt.syms.get_node_inputs(component_id).clone();

            // check inputs and node_inputs have the same length
            if inputs.len() != component_inputs.len() {
                let error = Error::ArityMismatch {
                    input_count: inputs.len(),
                    arity: component_inputs.len(),
                    location: ctxt.loc.clone(),
                };
                ctxt.errors.push(error);
                return Err(TerminationError);
            }

            // transform inputs and map then to the identifiers of the component inputs
            let inputs = inputs
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
