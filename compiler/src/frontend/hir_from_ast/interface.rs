use crate::ast::interface::{
    ComponentCall, FlowDeclaration, FlowExport, FlowExpression, FlowImport, FlowInstanciation,
    FlowKind, FlowStatement, Merge, Sample, Zip,
};
use crate::common::location::Location;
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::{
    flow_expression::{
        FlowExpression as HIRFlowExpression, FlowExpressionKind as HIRFlowExpressionKind,
    },
    flow_statement::{
        FlowDeclaration as HIRFlowDeclaration, FlowExport as HIRFlowExport,
        FlowImport as HIRFlowImport, FlowInstanciation as HIRFlowInstanciation,
        FlowStatement as HIRFlowStatement,
    },
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
                kind,
                typed_ident,
                eq_token,
                flow_expression,
                semi_token,
            }) => {
                let name = typed_ident.left.to_string();
                let flow_type = {
                    let inner = typed_ident
                        .right
                        .hir_from_ast(&location, symbol_table, errors)?;
                    match kind {
                        FlowKind::Signal(_) => Type::Signal(Box::new(inner)),
                        FlowKind::Event(_) => Type::Event(Box::new(inner)),
                    }
                };
                let id = symbol_table.insert_flow(
                    name,
                    None,
                    flow_type.clone(),
                    true,
                    location.clone(),
                    errors,
                )?;
                let flow_expression = flow_expression.hir_from_ast(symbol_table, errors)?;

                Ok(HIRFlowStatement::Declaration(HIRFlowDeclaration {
                    let_token,
                    kind,
                    id,
                    colon_token: typed_ident.colon,
                    flow_type,
                    eq_token,
                    flow_expression,
                    semi_token,
                }))
            }
            FlowStatement::Instanciation(FlowInstanciation {
                ident,
                eq_token,
                flow_expression,
                semi_token,
            }) => {
                // identifiers are already in symbol table because of flow export
                let name = ident.to_string();
                // get flow id
                let id = symbol_table.get_flow_id(&name, true, location.clone(), errors)?;
                // transform the expression
                let flow_expression = flow_expression.hir_from_ast(symbol_table, errors)?;

                Ok(HIRFlowStatement::Instanciation(HIRFlowInstanciation {
                    id,
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
                    kind,
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
                    kind,
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

impl Zip {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let Zip {
            flow_expression_1,
            flow_expression_2,
            ..
        } = self;
        Ok(HIRFlowExpressionKind::Zip {
            flow_expression_1: Box::new(flow_expression_1.hir_from_ast(symbol_table, errors)?),
            flow_expression_2: Box::new(flow_expression_2.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl Merge {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        let Merge {
            flow_expression_1,
            flow_expression_2,
            ..
        } = self;
        Ok(HIRFlowExpressionKind::Merge {
            flow_expression_1: Box::new(flow_expression_1.hir_from_ast(symbol_table, errors)?),
            flow_expression_2: Box::new(flow_expression_2.hir_from_ast(symbol_table, errors)?),
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
            ident_signal,
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

        // get output signal id
        let signal_name = ident_signal.unwrap().1.to_string();
        let (_, signal_id) = *symbol_table
            .get_node_outputs(component_id)
            .iter()
            .find(|(_, output_id)| symbol_table.get_name(*output_id) == &signal_name)
            .ok_or_else(|| {
                let error = Error::UnknownOuputSignal {
                    node_name: name,
                    signal_name,
                    location: location.clone(),
                };
                errors.push(error);
                TerminationError
            })?;

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
            signal_id,
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
            FlowExpression::Sample(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::Merge(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::Zip(flow_expression) => {
                flow_expression.hir_from_ast(symbol_table, errors)?
            }
            FlowExpression::ComponentCall(flow_expression) => {
                flow_expression.hir_from_ast(&location, symbol_table, errors)?
            }
            FlowExpression::Ident(ident) => {
                let name = ident.to_string();
                let id = symbol_table.get_flow_id(&name, false, location.clone(), errors)?;
                HIRFlowExpressionKind::Ident { id }
            }
        };
        Ok(HIRFlowExpression {
            kind,
            typing: None,
            location,
        })
    }
}
