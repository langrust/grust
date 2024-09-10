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

use super::{HIRFromAST, SimpleCtxt};

impl<'a> HIRFromAST<SimpleCtxt<'a>> for Service {
    type HIR = hir::Service;

    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
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
                    .hir_from_ast(ctxt)
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

impl<'a> HIRFromAST<SimpleCtxt<'a>> for FlowImport {
    type HIR = HIRFlowImport;

    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
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
            let inner = typed_path
                .right
                .hir_from_ast(&mut ctxt.add_loc(&location))?;
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

impl<'a> HIRFromAST<SimpleCtxt<'a>> for FlowExport {
    type HIR = HIRFlowExport;

    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
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
            let inner = typed_path
                .right
                .hir_from_ast(&mut ctxt.add_loc(&location))?;
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

impl<'a> HIRFromAST<SimpleCtxt<'a>> for FlowStatement {
    type HIR = HIRFlowStatement;

    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                let_token,
                typed_pattern,
                eq_token,
                flow_expression,
                semi_token,
            }) => {
                let pattern = typed_pattern.hir_from_ast(ctxt)?;
                let flow_expression =
                    flow_expression.hir_from_ast(&mut ctxt.add_loc(&Location::default()))?;

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
                let pattern = pattern.hir_from_ast(ctxt)?;
                // transform the expression
                let flow_expression =
                    flow_expression.hir_from_ast(&mut ctxt.add_loc(&Location::default()))?;

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

impl<'a> HIRFromAST<SimpleCtxt<'a>> for FlowPattern {
    type HIR = hir::stmt::Pattern;

    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
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
                let inner = ty.hir_from_ast(&mut ctxt.add_loc(&location))?;
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
                    .map(|pattern| pattern.hir_from_ast(ctxt))
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
        frontend::hir_from_ast::{HIRFromAST, LocCtxt},
        hir::flow,
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for ast::interface::Sample {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
            let Sample {
                flow_expression,
                period_ms,
                ..
            } = self;
            Ok(hir::flow::Kind::Sample {
                flow_expression: Box::new(flow_expression.hir_from_ast(ctxt)?),
                period_ms: period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for Scan {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
            let Scan {
                flow_expression,
                period_ms,
                ..
            } = self;
            Ok(hir::flow::Kind::Scan {
                flow_expression: Box::new(flow_expression.hir_from_ast(ctxt)?),
                period_ms: period_ms.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for Timeout {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
            let Timeout {
                flow_expression,
                deadline,
                ..
            } = self;
            Ok(hir::flow::Kind::Timeout {
                flow_expression: Box::new(flow_expression.hir_from_ast(ctxt)?),
                deadline: deadline.base10_parse().unwrap(),
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for Throttle {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
            let Throttle {
                flow_expression,
                delta,
                ..
            } = self;
            Ok(hir::flow::Kind::Throttle {
                flow_expression: Box::new(flow_expression.hir_from_ast(ctxt)?),
                delta,
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for OnChange {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
            let OnChange {
                flow_expression, ..
            } = self;
            Ok(hir::flow::Kind::OnChange {
                flow_expression: Box::new(flow_expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for Merge {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
            let Merge {
                flow_expression_1,
                flow_expression_2,
                ..
            } = self;
            Ok(hir::flow::Kind::Merge {
                flow_expression_1: Box::new(flow_expression_1.hir_from_ast(ctxt)?),
                flow_expression_2: Box::new(flow_expression_2.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for ComponentCall {
        type HIR = hir::flow::Kind;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<hir::flow::Kind> {
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
                let error = Error::IncompatibleInputsNumber {
                    given_inputs_number: inputs.len(),
                    expected_inputs_number: component_inputs.len(),
                    location: ctxt.loc.clone(),
                };
                ctxt.errors.push(error);
                return Err(TerminationError);
            }

            // transform inputs and map then to the identifiers of the component inputs
            let inputs = inputs
                .into_iter()
                .zip(component_inputs)
                .map(|(input, id)| Ok((id, input.hir_from_ast(ctxt)?)))
                .collect::<TRes<Vec<_>>>()?;

            Ok(hir::flow::Kind::ComponentCall {
                component_id,
                inputs,
            })
        }
    }

    impl<'a> HIRFromAST<LocCtxt<'a>> for FlowExpression {
        type HIR = flow::Expr;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression and check identifiers good use
        fn hir_from_ast(self, ctxt: &mut LocCtxt<'a>) -> TRes<Self::HIR> {
            let kind = match self {
                FlowExpression::Ident(ident) => {
                    let id = ctxt
                        .syms
                        .get_flow_id(&ident, false, ctxt.loc.clone(), ctxt.errors)?;
                    flow::Kind::Ident { id }
                }
                FlowExpression::ComponentCall(flow_expression) => {
                    flow_expression.hir_from_ast(ctxt)?
                }
                FlowExpression::Sample(flow_expression) => flow_expression.hir_from_ast(ctxt)?,
                FlowExpression::Scan(flow_expression) => flow_expression.hir_from_ast(ctxt)?,
                FlowExpression::Timeout(flow_expression) => flow_expression.hir_from_ast(ctxt)?,
                FlowExpression::Throttle(flow_expression) => flow_expression.hir_from_ast(ctxt)?,
                FlowExpression::OnChange(flow_expression) => flow_expression.hir_from_ast(ctxt)?,
                FlowExpression::Merge(flow_expression) => flow_expression.hir_from_ast(ctxt)?,
            };
            Ok(flow::Expr {
                kind,
                typing: None,
                location: ctxt.loc.clone(),
            })
        }
    }
}
