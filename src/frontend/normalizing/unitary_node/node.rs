use petgraph::algo::has_path_connecting;

use crate::error::{Error, TerminationError};
use crate::hir::contract::Term;
use crate::hir::{memory::Memory, node::Node, unitary_node::UnitaryNode};
use crate::symbol_table::SymbolTable;
use crate::{common::scope::Scope, hir::contract::Contract};

impl Node {
    /// Change every node application into unitary node application.
    ///
    /// It removes unused inputs from unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    /// ```
    ///
    /// The application of the node `my_node(g-1, v).o2` is changed
    /// to the application of the unitary node `my_node(v).o2`
    pub fn change_node_application_into_unitary_node_application(
        &mut self,
        symbol_table: &SymbolTable,
    ) {
        self.unitary_nodes.values_mut().for_each(|unitary_node| {
            unitary_node.statements.iter_mut().for_each(|equation| {
                equation
                    .expression
                    .change_node_application_into_unitary_node_application(symbol_table)
            })
        })
    }

    /// Generate unitary nodes from mother node.
    ///
    /// Generate and add unitary nodes to mother node.
    /// Unitary nodes are nodes with one output and contains
    /// all signals from which the output computation depends.
    ///
    /// It also detects unused signal definitions or inputs.
    pub fn generate_unitary_nodes(
        &mut self,
        symbol_table: &mut SymbolTable,
        creusot_contract: bool,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // get outputs identifiers
        let outputs = self
            .unscheduled_equations
            .values()
            .filter(|equation| symbol_table.get_scope(equation.id).eq(&Scope::Output))
            .map(|equation| equation.id.clone())
            .collect::<Vec<_>>();

        // construct unitary node for each output and get used signals
        let used_signals = outputs
            .into_iter()
            .flat_map(|output| self.add_unitary_node(output, symbol_table, creusot_contract))
            .collect::<Vec<_>>();

        // check that every signals are used
        let graph = self
            .graph
            .get()
            .expect("node dependency graph should be computed");
        let unused_signals = graph
            .nodes()
            .filter(|id| !used_signals.contains(id))
            .collect::<Vec<_>>();
        unused_signals
            .into_iter()
            .map(|id| {
                let error = Error::UnusedSignal {
                    node: symbol_table.get_name(self.id).clone(),
                    signal: symbol_table.get_name(id).clone(),
                    location: self.location.clone(),
                };
                errors.push(error);
                Err(TerminationError)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<_, _>>()
    }

    fn add_unitary_node(
        &mut self,
        output: usize,
        symbol_table: &mut SymbolTable,
        _creusot_contract: bool,
    ) -> Vec<usize> {
        let Node {
            id: node,
            unscheduled_equations,
            unitary_nodes,
            contract:
                Contract {
                    requires,
                    ensures,
                    invariant,
                },
            location,
            ..
        } = self;

        // construct unitary node's subgraph from its output
        let graph = self
            .graph
            .get()
            .expect("node dependency graph should be computed");
        let mut subgraph = graph.clone();
        graph.nodes().for_each(|id| {
            let has_path = has_path_connecting(graph, output, id, None); // TODO: contrary?
            if !has_path {
                subgraph.remove_node(id);
            }
        });

        // get useful inputs (in application order)
        let unitary_node_inputs = symbol_table
            .get_node_inputs(*node)
            .iter()
            .filter(|id| subgraph.contains_node(**id))
            .map(|id| *id)
            .collect::<Vec<_>>();

        // retrieve statements from useful signals
        let statements = subgraph
            .nodes()
            .filter_map(|signal| unscheduled_equations.get(&signal))
            .cloned()
            .collect();

        // retrieve contract from usefull signals
        let retrieve_terms = |terms: &Vec<Term>| {
            terms
                .iter()
                .filter_map(|term| {
                    if subgraph.nodes().any(|id| term.contains_id(id)) {
                        Some(term)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        let contract = Contract {
            requires: retrieve_terms(requires),
            ensures: retrieve_terms(ensures),
            invariant: retrieve_terms(invariant),
        };

        let id = symbol_table.insert_unitary_node(
            symbol_table.get_name(*node).clone(),
            symbol_table.get_name(output).clone(),
            symbol_table.is_component(*node),
            *node,
            unitary_node_inputs,
            output,
        );

        let used_signals = subgraph.nodes().collect::<Vec<_>>();

        // construct unitary node
        let unitary_node = UnitaryNode {
            id,
            contract,
            statements,
            memory: Memory::new(),
            location: location.clone(),
            graph: subgraph,
        };
        // insert it in node's storage
        unitary_nodes.insert(output, unitary_node);

        used_signals
    }
}
