use crate::ast::interface::{
    ComponentCall, FlowDeclaration, FlowExport, FlowExpression, FlowImport, FlowInstanciation,
    FlowKind, FlowPattern, FlowStatement, OnChange, Sample, Scan, Throtle, Timeout,
};
use crate::common::location::Location;
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::pattern::PatternKind;
use crate::hir::{
    flow_expression::{
        FlowExpression as HIRFlowExpression, FlowExpressionKind as HIRFlowExpressionKind,
    },
    interface::{
        FlowDeclaration as HIRFlowDeclaration, FlowExport as HIRFlowExport,
        FlowImport as HIRFlowImport, FlowInstanciation as HIRFlowInstanciation,
        FlowStatement as HIRFlowStatement,
    },
    pattern::Pattern,
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for FlowStatement {
    type HIR = HIRFlowStatement;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
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
            FlowStatement::Instanciation(FlowInstanciation {
                pattern,
                eq_token,
                flow_expression,
                semi_token,
            }) => {
                // transform pattern and check its identifiers exist
                let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                // transform the expression
                let flow_expression = flow_expression.hir_from_ast(symbol_table, errors)?;

                Ok(HIRFlowStatement::Instanciation(HIRFlowInstanciation {
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
                        FlowKind::Signal(_) => Type::Signal(Box::new(inner)),
                        FlowKind::Event(_) => Type::Event(Box::new(inner)),
                    }
                };
                let id = symbol_table.insert_flow(
                    name,
                    Some(path.clone()),
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
                        FlowKind::Signal(_) => Type::Signal(Box::new(inner)),
                        FlowKind::Event(_) => Type::Event(Box::new(inner)),
                    }
                };
                let id = symbol_table.insert_flow(
                    name,
                    Some(path.clone()),
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
    ) -> Result<Self::HIR, TerminationError> {
        let location = Location::default();

        match self {
            FlowPattern::Single {
                kind, ident, ty, ..
            } => {
                let typing = ty.hir_from_ast(&location, symbol_table, errors)?;
                let flow_typing = match kind {
                    FlowKind::Signal(_) => Type::Signal(Box::new(typing)),
                    FlowKind::Event(_) => Type::Event(Box::new(typing)),
                };
                let id = symbol_table.insert_identifier(
                    ident.to_string(),
                    Some(flow_typing.clone()),
                    true,
                    location.clone(),
                    errors,
                )?;

                Ok(Pattern {
                    kind: PatternKind::Typed {
                        pattern: Box::new(Pattern {
                            kind: PatternKind::Identifier { id },
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
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                let types = elements
                    .iter()
                    .map(|pattern| pattern.typing.as_ref().unwrap().clone())
                    .collect();
                Ok(Pattern {
                    kind: PatternKind::Tuple { elements },
                    typing: Some(Type::Tuple(types)),
                    location,
                })
            }
        }
    }
}

impl Sample {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let Sample {
            flow_expression,
            period_ms,
            ..
        } = self;
        Ok(HIRFlowExpressionKind::Sample {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            period_ms: period_ms.base10_parse().unwrap(),
        })
    }
}

impl Scan {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let Scan {
            flow_expression,
            period_ms,
            ..
        } = self;
        Ok(HIRFlowExpressionKind::Scan {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            period_ms: period_ms.base10_parse().unwrap(),
        })
    }
}

impl Timeout {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let Timeout {
            flow_expression,
            deadline,
            ..
        } = self;
        Ok(HIRFlowExpressionKind::Timeout {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            deadline: deadline.base10_parse().unwrap(),
        })
    }
}

impl Throtle {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let Throtle {
            flow_expression,
            delta,
            ..
        } = self;
        Ok(HIRFlowExpressionKind::Throtle {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            delta,
        })
    }
}

impl OnChange {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let OnChange {
            flow_expression, ..
        } = self;
        Ok(HIRFlowExpressionKind::OnChange {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl ComponentCall {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let ComponentCall {
            ident_component,
            inputs,
            ..
        } = self;

        let name = ident_component.to_string();

        // get called component id
        let component_id = symbol_table.get_node_id(&name, false, location.clone(), errors)?;

        // if not component raise error: only component can be called in interface
        if !symbol_table.is_component(component_id) {
            let error = Error::NodeCall {
                name,
                location: location.clone(),
            };
            errors.push(error);
            return Err(TerminationError);
        }

        let component_inputs = symbol_table.get_node_inputs(component_id).clone();

        // check inputs and node_inputs have the same length
        if inputs.len() != component_inputs.len() {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: inputs.len(),
                expected_inputs_number: component_inputs.len(),
                location: location.clone(),
            };
            errors.push(error);
            return Err(TerminationError);
        }

        // transform inputs and map then to the identifiers of the component inputs
        let inputs = inputs
            .into_iter()
            .zip(component_inputs)
            .map(|(input, id)| Ok((id, input.hir_from_ast(symbol_table, errors)?)))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(HIRFlowExpressionKind::ComponentCall {
            component_id,
            inputs,
        })
    }
}

impl HIRFromAST for FlowExpression {
    type HIR = HIRFlowExpression;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let location = Location::default();
        let kind = match self {
            FlowExpression::Ident(ident) => {
                let name = ident.to_string();
                let id = symbol_table.get_flow_id(&name, false, location.clone(), errors)?;
                HIRFlowExpressionKind::Ident { id }
            }
            FlowExpression::ComponentCall(flow_expression) => {
                flow_expression.hir_from_ast(&location, symbol_table, errors)?
            }
            FlowExpression::Sample(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::Scan(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::Timeout(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::Throtle(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::OnChange(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
        };
        Ok(HIRFlowExpression {
            kind,
            typing: None,
            location,
        })
    }
}
