use std::collections::BTreeMap;

use crate::ast::node::Node;
use crate::common::scope::Scope;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node as HIRNode, once_cell::OnceCell};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Node {
    type HIR = HIRNode;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Node {
            id,
            equations,
            contract,
            location,
            ..
        } = self;

        let id = symbol_table.get_node_id(&id, false, location.clone(), errors)?;

        // create local context with all signals
        symbol_table.local();
        symbol_table.restore_context(id);

        let unscheduled_equations = equations
            .into_iter()
            .map(|(signal, equation)| {
                let id =
                    symbol_table.get_signal_id(&signal, true, equation.location.clone(), errors)?;
                Ok((id, equation.hir_from_ast(symbol_table, errors)?))
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let contract = contract.hir_from_ast(symbol_table, errors)?;

        symbol_table.global();

        Ok(HIRNode {
            id,
            unscheduled_equations,
            unitary_nodes: BTreeMap::new(),
            contract,
            location,
            graph: OnceCell::new(),
        })
    }
}

impl Node {
    /// Store node's identifiers in symbol table.
    pub fn store(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        symbol_table.local();

        let inputs = self
            .inputs
            .iter()
            .map(|(name, typing)| {
                let typing = typing
                    .clone()
                    .hir_from_ast(&self.location, symbol_table, errors)?;
                let id = symbol_table.insert_signal(
                    name.clone(),
                    Scope::Input,
                    Some(typing),
                    true,
                    self.location.clone(),
                    errors,
                )?;
                Ok(id)
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let outputs = self
            .equations
            .iter()
            .filter(|(_, equation)| Scope::Output == equation.scope)
            .map(|(name, equation)| {
                let typing = equation.signal_type.clone().hir_from_ast(
                    &self.location,
                    symbol_table,
                    errors,
                )?;
                let id = symbol_table.insert_signal(
                    name.clone(),
                    Scope::Output,
                    Some(typing),
                    true,
                    equation.location.clone(),
                    errors,
                )?;
                Ok((name.clone(), id))
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        let locals = self
            .equations
            .iter()
            .filter(|(_, equation)| Scope::Local == equation.scope)
            .map(|(name, equation)| {
                let typing = equation.signal_type.clone().hir_from_ast(
                    &self.location,
                    symbol_table,
                    errors,
                )?;
                let id = symbol_table.insert_signal(
                    name.clone(),
                    Scope::Local,
                    Some(typing),
                    true,
                    equation.location.clone(),
                    errors,
                )?;
                Ok((name.clone(), id))
            })
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        symbol_table.global();

        let _ = symbol_table.insert_node(
            self.id.clone(),
            self.is_component,
            false,
            inputs,
            outputs,
            locals,
            self.location.clone(),
            errors,
        )?;

        Ok(())
    }
}
