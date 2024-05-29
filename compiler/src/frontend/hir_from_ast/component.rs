use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::ast::colon::Colon;
use crate::ast::component::Component;
use crate::common::convert_case::camel_case;
use crate::common::location::Location;
use crate::common::scope::Scope;
use crate::error::{Error, TerminationError};
use crate::hir::memory::Memory;
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

        let statements = equations
            .into_iter()
            .map(|equation| equation.hir_from_ast(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let contract = contract.hir_from_ast(symbol_table, errors)?;

        symbol_table.global();

        Ok(HIRNode {
            id,
            statements,
            contract,
            location,
            graph: DiGraphMap::new(),
            memory: Memory::new(),
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
        let period = self
            .period
            .as_ref()
            .map(|(_, literal, _)| literal.base10_parse().unwrap());
        let location = Location::default();

        // store input signals and get their ids
        let inputs = self
            .args
            .iter()
            .filter(|Colon { right: typing, .. }| !typing.is_event())
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

        // store input events as element of an "event enumeration"
        let enum_name = camel_case(&format!("{name}Event"));
        let mut element_ids = self
            .args
            .iter()
            .filter(|Colon { right: typing, .. }| typing.is_event())
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
                    let id = symbol_table.insert_event_element(
                        name,
                        enum_name.clone(),
                        typing,
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

        let events = if !element_ids.is_empty() {
            // create enumeration of events
            let id = symbol_table.insert_event_enum(
                enum_name.clone(),
                element_ids.clone(),
                true,
                location.clone(),
                errors,
            )?;
            element_ids.push(id);

            // create identifier for event
            let event_name = format!("{name}_event");
            let id =
                symbol_table.insert_event(event_name.clone(), true, location.clone(), errors)?;
            element_ids.push(id);

            element_ids
        } else {
            vec![]
        };

        // store outputs and get their ids
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
            .collect::<Result<Vec<_>, _>>()?;

        // store locals and get their ids
        let locals = self
            .equations
            .iter()
            .filter_map(|equation| equation.store_local_declarations(symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<HashMap<String, usize>>();

        symbol_table.global();

        let _ = symbol_table.insert_node(
            name, false, inputs, events, outputs, locals, period, location, errors,
        )?;

        Ok(())
    }
}
