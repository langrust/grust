use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::ast::colon::Colon;
use crate::ast::component::Component;
use crate::ast::equation::Equation;
use crate::common::location::Location;
use crate::common::scope::Scope;
use crate::error::{Error, TerminationError};
use crate::hir::node::Node as HIRNode;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Component {
    type HIR = HIRNode;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Component {
            ident,
            contract,
            equations,
            ..
        } = self;
        let name = ident.to_string();
        let location = Location::default();
        let id = symbol_table.get_node_id(&name, false, location.clone(), errors)?;

        // create local context with all signals
        symbol_table.local();
        symbol_table.restore_context(id);

        let unscheduled_equations = equations
            .into_iter()
            .map(|equation| equation.hir_from_ast(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let contract = contract.hir_from_ast(symbol_table, errors)?;

        symbol_table.global();

        Ok(HIRNode {
            id,
            unscheduled_equations,
            unitary_nodes: HashMap::new(),
            contract,
            location,
            graph: DiGraphMap::new(),
        })
    }
}

impl Component {
    /// Store node's signals in symbol table.
    pub fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        symbol_table.local();

        let name = self.ident.to_string();
        let is_component = false;
        let period = self
            .period
            .as_ref()
            .map(|(_, literal, _)| literal.base10_parse().unwrap());
        let location = Location::default();
        let inputs = self
            .args
            .iter()
            .map(
                |Colon {
                     left: ident,
                     right: typing,
                     ..
                 }| {
                    let name = ident.to_string();
                    let typing = typing
                        .clone()
                        .hir_from_ast(&location, symbol_table, errors)?;
                    let id = symbol_table.insert_signal(
                        name,
                        Scope::Input,
                        Some(typing),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    Ok(id)
                },
            )
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let outputs = self
            .outs
            .iter()
            .map(
                |Colon {
                     left: ident,
                     right: typing,
                     ..
                 }| {
                    let name = ident.to_string();
                    let typing = typing
                        .clone()
                        .hir_from_ast(&location, symbol_table, errors)?;
                    let id = symbol_table.insert_signal(
                        name.clone(),
                        Scope::Output,
                        Some(typing),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    Ok((name, id))
                },
            )
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<HashMap<_, _>, _>>()?;

        let locals = self
            .equations
            .iter()
            .filter_map(|equation| match equation {
                Equation::LocalDef(declaration) => {
                    Some(declaration.typed_pattern.store(symbol_table, errors))
                }
                Equation::OutputDef(_) => None,
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<HashMap<String, usize>>();

        symbol_table.global();

        let _ = symbol_table.insert_node(
            name,
            is_component,
            false,
            inputs,
            outputs,
            locals,
            period,
            location,
            errors,
        )?;

        Ok(())
    }
}
