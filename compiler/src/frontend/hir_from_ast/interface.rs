prelude! {
    ast::interface::{
        Constrains, FlowDeclaration, FlowExport, FlowExpression, FlowImport,
        FlowInstantiation, FlowKind, FlowPattern, FlowStatement, Service,
    },
    hir::{
        Pattern, flow,
        interface::{
            FlowDeclaration as HIRFlowDeclaration, FlowExport as HIRFlowExport,
            FlowImport as HIRFlowImport, FlowInstantiation as HIRFlowInstantiation,
            FlowStatement as HIRFlowStatement,
        },
    },
}

use super::HIRFromAST;

impl HIRFromAST for Service {
    type HIR = hir::Service;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let Service {
            ident,
            constrains,
            flow_statements,
            ..
        } = self;

        let id =
            symbol_table.insert_service(ident.to_string(), true, Location::default(), errors)?;

        let constrains = if let Some(Constrains { min, max, .. }) = constrains {
            (min.base10_parse().unwrap(), max.base10_parse().unwrap())
        } else {
            (10, 500)
        };

        symbol_table.local();
        let statements = flow_statements
            .into_iter()
            .map(|flow_statement| {
                flow_statement
                    .hir_from_ast(symbol_table, errors)
                    .map(|res| (symbol_table.get_fresh_id(), res))
            })
            .collect::<TRes<HashMap<_, _>>>()?;
        let graph = Default::default();
        symbol_table.global();

        Ok(hir::Service {
            id,
            constrains,
            statements,
            graph,
        })
    }
}

impl HIRFromAST for FlowImport {
    type HIR = HIRFlowImport;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
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
                .hir_from_ast(&location, symbol_table, errors)?;
            match kind {
                FlowKind::Signal(_) => Typ::signal(inner),
                FlowKind::Event(_) => Typ::event(inner),
            }
        };
        let id = symbol_table.insert_flow(
            name,
            Some(path.clone()),
            kind,
            flow_type.clone(),
            true,
            location.clone(),
            errors,
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

impl HIRFromAST for FlowExport {
    type HIR = HIRFlowExport;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
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
                .hir_from_ast(&location, symbol_table, errors)?;
            match kind {
                FlowKind::Signal(_) => Typ::signal(inner),
                FlowKind::Event(_) => Typ::event(inner),
            }
        };
        let id = symbol_table.insert_flow(
            name,
            Some(path.clone()),
            kind,
            flow_type.clone(),
            true,
            location.clone(),
            errors,
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

impl HIRFromAST for FlowStatement {
    type HIR = HIRFlowStatement;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                let_token,
                typed_pattern,
                eq_token,
                flow_expression,
                semi_token,
            }) => {
                let pattern = typed_pattern.hir_from_ast(symbol_table, errors)?;
                let flow_expression = flow_expression.hir_from_ast(symbol_table, errors)?;

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
                let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                // transform the expression
                let flow_expression = flow_expression.hir_from_ast(symbol_table, errors)?;

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

impl HIRFromAST for FlowPattern {
    type HIR = Pattern;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();

        match self {
            FlowPattern::Single { ident } => {
                let id = symbol_table.get_flow_id(
                    &ident.to_string(),
                    false,
                    location.clone(),
                    errors,
                )?;
                let typing = symbol_table.get_type(id);

                Ok(Pattern {
                    kind: hir::pattern::Kind::Identifier { id },
                    typing: Some(typing.clone()),
                    location,
                })
            }
            FlowPattern::SingleTyped {
                kind, ident, ty, ..
            } => {
                let inner = ty.hir_from_ast(&location, symbol_table, errors)?;
                let flow_typing = match kind {
                    FlowKind::Signal(_) => Typ::signal(inner),
                    FlowKind::Event(_) => Typ::event(inner),
                };
                let id = symbol_table.insert_flow(
                    ident.to_string(),
                    None,
                    kind,
                    flow_typing.clone(),
                    true,
                    location.clone(),
                    errors,
                )?;

                Ok(Pattern {
                    kind: hir::pattern::Kind::Typed {
                        pattern: Box::new(Pattern {
                            kind: hir::pattern::Kind::Identifier { id },
                            typing: Some(flow_typing.clone()),
                            location: location.clone(),
                        }),
                        typing: flow_typing.clone(),
                    },
                    typing: Some(flow_typing),
                    location,
                })
            }
            FlowPattern::Tuple { patterns, .. } => {
                let elements = patterns
                    .into_iter()
                    .map(|pattern| pattern.hir_from_ast(symbol_table, errors))
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
                Ok(Pattern {
                    kind: hir::pattern::Kind::Tuple { elements },
                    typing: Some(ty),
                    location,
                })
            }
        }
    }
}

impl HIRFromAST for FlowExpression {
    type HIR = flow::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
        let loc = &location;
        let kind = match self {
            FlowExpression::Ident(ident) => {
                let id = symbol_table.get_flow_id(&ident, false, location.clone(), errors)?;
                flow::Kind::Ident { id }
            }
            FlowExpression::ComponentCall(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
            FlowExpression::Sample(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
            FlowExpression::Scan(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
            FlowExpression::Timeout(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
            FlowExpression::Throttle(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
            FlowExpression::OnChange(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
            FlowExpression::Merge(flow_expression) => {
                flow_expression.hir_from_ast(loc, symbol_table, errors)?
            }
        };
        Ok(flow::Expr {
            kind,
            typing: None,
            location,
        })
    }
}
