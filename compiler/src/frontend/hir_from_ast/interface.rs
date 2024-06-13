prelude! {
    ast::interface::{
        FlowDeclaration, FlowExport, FlowExpression, FlowImport, FlowInstantiation,
        FlowKind, FlowPattern, FlowStatement,
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

impl HIRFromAST for FlowStatement {
    type HIR = HIRFlowStatement;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
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
            FlowStatement::Import(FlowImport {
                import_token,
                kind,
                mut typed_path,
                semi_token,
            }) => {
                let last = typed_path.left.segments.pop().unwrap().into_value();
                let name = last.ident.to_string();
                assert!(last.arguments.is_none());
                let path = typed_path.left;
                let flow_type = {
                    let inner = typed_path
                        .right
                        .hir_from_ast(&location, symbol_table, errors)?;
                    match kind {
                        FlowKind::Signal(_) => Typ::Signal(Box::new(inner)),
                        FlowKind::Event(_) => Typ::Event(Box::new(inner)),
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
                Ok(HIRFlowStatement::Import(HIRFlowImport {
                    import_token,
                    id,
                    path,
                    colon_token: typed_path.colon,
                    flow_type,
                    semi_token,
                }))
            }
            FlowStatement::Export(FlowExport {
                export_token,
                kind,
                mut typed_path,
                semi_token,
            }) => {
                let last = typed_path.left.segments.pop().unwrap().into_value();
                let name = last.ident.to_string();
                assert!(last.arguments.is_none());
                let path = typed_path.left;
                let flow_type = {
                    let inner = typed_path
                        .right
                        .hir_from_ast(&location, symbol_table, errors)?;
                    match kind {
                        FlowKind::Signal(_) => Typ::Signal(Box::new(inner)),
                        FlowKind::Event(_) => Typ::Event(Box::new(inner)),
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
                Ok(HIRFlowStatement::Export(HIRFlowExport {
                    export_token,
                    id,
                    path,
                    colon_token: typed_path.colon,
                    flow_type,
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
                let typing = ty.hir_from_ast(&location, symbol_table, errors)?;
                let flow_typing = match kind {
                    FlowKind::Signal(_) => Typ::Signal(Box::new(typing)),
                    FlowKind::Event(_) => Typ::Event(Box::new(typing)),
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

                let types = elements
                    .iter()
                    .map(|pattern| pattern.typing.as_ref().unwrap().clone())
                    .collect();
                Ok(Pattern {
                    kind: hir::pattern::Kind::Tuple { elements },
                    typing: Some(Typ::Tuple(types)),
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
                let name = ident.to_string();
                let id = symbol_table.get_flow_id(&name, false, location.clone(), errors)?;
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
        };
        Ok(flow::Expr {
            kind,
            typing: None,
            location,
        })
    }
}
