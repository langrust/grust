use crate::ast::interface::{
    FlowExpression, FlowExpressionKind, FlowStatement, FlowType, Interface,
};
use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::hir::interface::{
    FlowExpression as HIRFlowExpression, FlowExpressionKind as HIRFlowExpressionKind,
    FlowStatement as HIRFlowStatement, Interface as HIRInterface,
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Interface {
    type HIR = HIRInterface;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Interface {
            id,
            flow_statements,
            imports,
            exports,
            location,
        } = self;

        symbol_table.local();

        // store imports
        let imports = imports
            .into_iter()
            .map(|(ty, path)| {
                let name = path.get_name();
                let typing = ty.hir_from_ast(&location, symbol_table, errors)?;
                symbol_table.insert_flow(name, Some(path), typing, true, location.clone(), errors)
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        // store flows and chack identifier usage in statements
        let flow_statements = flow_statements
            .into_iter()
            .map(|statement| statement.hir_from_ast(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        // set path to export flows
        let exports = exports
            .into_iter()
            .map(|path| {
                // exports are already in symbol table
                // because of flow statements
                let name = path.get_name();
                // get flow id
                let id = symbol_table.get_flow_id(&name, true, location.clone(), errors)?;
                // set export path
                symbol_table.set_path(id, path);

                Ok(id)
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        symbol_table.global();

        // store interface
        let id =
            symbol_table.insert_interface(id, false, imports, exports, location.clone(), errors)?;

        Ok(HIRInterface {
            id,
            flow_statements,
        })
    }
}

impl FlowType {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<FlowType, TerminationError> {
        // precondition: Typedefs are stored in symbol table
        // postcondition: construct a new FlowType without `Type::NotDefinedYet`
        match self {
            FlowType::Signal(typing) => Ok(FlowType::Signal(typing.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?)),
            FlowType::Event(typing) => Ok(FlowType::Event(typing.hir_from_ast(
                location,
                symbol_table,
                errors,
            )?)),
        }
    }
}

impl HIRFromAST for FlowStatement {
    type HIR = HIRFlowStatement;

    // precondition: interface imports are already stored in symbol table
    // postcondition: construct HIR interface and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let FlowStatement {
            ident,
            flow_type,
            flow_expression,
            location,
        } = self;

        let flow_type = flow_type
            .clone()
            .hir_from_ast(&location, symbol_table, errors)?;
        let id =
            symbol_table.insert_flow(ident, None, flow_type, true, location.clone(), errors)?;
        let flow_expression = flow_expression.hir_from_ast(symbol_table, errors)?;

        Ok(HIRFlowStatement {
            id,
            flow_expression,
        })
    }
}

impl FlowExpressionKind {
    /// Transforms AST into HIR and check identifiers good use.
    pub fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<HIRFlowExpressionKind, TerminationError> {
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        match self {
            FlowExpressionKind::Ident { ident } => {
                let id = symbol_table.get_flow_id(&ident, false, location.clone(), errors)?;
                Ok(HIRFlowExpressionKind::Ident { id })
            }
            FlowExpressionKind::Timeout {
                flow_expression,
                timeout_ms,
            } => Ok(HIRFlowExpressionKind::Timeout {
                flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
                timeout_ms,
            }),
            FlowExpressionKind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => Ok(HIRFlowExpressionKind::Merge {
                flow_expression_1: Box::new(flow_expression_1.hir_from_ast(symbol_table, errors)?),
                flow_expression_2: Box::new(flow_expression_2.hir_from_ast(symbol_table, errors)?),
            }),
            FlowExpressionKind::Zip {
                flow_expression_1,
                flow_expression_2,
            } => Ok(HIRFlowExpressionKind::Zip {
                flow_expression_1: Box::new(flow_expression_1.hir_from_ast(symbol_table, errors)?),
                flow_expression_2: Box::new(flow_expression_2.hir_from_ast(symbol_table, errors)?),
            }),
            FlowExpressionKind::ComponentCall {
                ident_component,
                inputs,
                ident_signal,
            } => {
                // get called component id
                let component_id =
                    symbol_table.get_node_id(&ident_component, false, location.clone(), errors)?;

                // if not component raise error: only component can be called in interface
                if !symbol_table.is_component(component_id) {
                    let error = Error::NodeCall {
                        name: ident_component.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                // get output signal id
                let signal_id = *symbol_table
                    .get_node_outputs(component_id)
                    .find(|output_id| symbol_table.get_name(**output_id) == &ident_signal)
                    .ok_or_else(|| {
                        let error = Error::UnknownOuputSignal {
                            node_name: ident_component,
                            signal_name: ident_signal,
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
        let FlowExpression { kind, location } = self;
        Ok(HIRFlowExpression {
            kind: kind.hir_from_ast(&location, symbol_table, errors)?,
            typing: None,
            location,
        })
    }
}
