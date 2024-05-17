use std::collections::HashMap;

use petgraph::{algo::all_simple_paths, graphmap::DiGraphMap};

use crate::{
    common::label::Label,
    hir::{
        expression::ExpressionKind,
        identifier_creator::IdentifierCreator,
        memory::Memory,
        node::Node,
        statement::Statement,
        stream_expression::{StreamExpression, StreamExpressionKind},
    },
    symbol_table::SymbolTable,
};

use super::Union;

impl Statement<StreamExpression> {
    /// Add the statement identifier to the identifier creator.
    ///
    /// It will add the statement identifier to the identifier creator.
    /// If the identifier already exists, then the new identifer created by
    /// the identifier creator will be added to the renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<usize, Union<usize, StreamExpression>>,
        symbol_table: &mut SymbolTable,
    ) {
        // create fresh identifiers for the new statement
        let local_signals = self.pattern.identifiers();
        for signal_id in local_signals {
            let name = symbol_table.get_name(signal_id);
            let fresh_name =
                identifier_creator.new_identifier(String::new(), name.clone(), String::new());
            if &fresh_name != name {
                // TODO: should we just replace anyway?
                let scope = symbol_table.get_scope(signal_id).clone();
                let typing = Some(symbol_table.get_type(signal_id).clone());
                let fresh_id = symbol_table.insert_fresh_signal(fresh_name, scope, typing);
                let test_first_insert = context_map.insert(signal_id, Union::I1(fresh_id));
                debug_assert!(test_first_insert.is_none());
            }
        }
    }

    /// Replace identifier occurence by element in context.
    ///
    /// It will return a new statement where the expression has been modified
    /// according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function
    /// with the statement `z = x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<usize, Union<usize, StreamExpression>>,
    ) -> Statement<StreamExpression> {
        let mut new_statement = self.clone();

        // replace statement's identifiers by the new ones
        let local_signals = new_statement.pattern.identifiers_mut();
        for signal_id in local_signals {
            if let Some(element) = context_map.get(&signal_id) {
                match element {
                    Union::I1(new_id)
                    | Union::I2(StreamExpression {
                        kind:
                            StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier { id: new_id },
                            },
                        ..
                    }) => {
                        *signal_id = new_id.clone();
                    }
                    Union::I2(_) => unreachable!(),
                }
            }
        }

        // replace identifiers in statement's expression
        new_statement.expression.replace_by_context(context_map);

        new_statement
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// ```
    /// In this example, an statement `fib: int = semi_fib(fib).o` calls
    /// `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed_reccursive(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &mut SymbolTable,
        nodes: &HashMap<usize, Node>,
    ) -> Vec<Statement<StreamExpression>> {
        let mut current_statements = vec![self.clone()];
        let mut new_statements =
            self.inline_when_needed(memory, identifier_creator, graph, symbol_table, nodes);
        while current_statements != new_statements {
            current_statements = new_statements;
            new_statements = current_statements
                .clone()
                .into_iter()
                .flat_map(|statement| {
                    statement.inline_when_needed(
                        memory,
                        identifier_creator,
                        graph,
                        symbol_table,
                        nodes,
                    )
                })
                .collect();
        }
        new_statements
    }

    fn inline_when_needed(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        graph: &DiGraphMap<usize, Label>,
        symbol_table: &mut SymbolTable,
        nodes: &HashMap<usize, Node>,
    ) -> Vec<Statement<StreamExpression>> {
        match &self.expression.kind {
            StreamExpressionKind::UnitaryNodeApplication {
                node_id, inputs, ..
            } => {
                // a loop in the graph induces that "node call" inputs depends on output
                let signals = self.pattern.identifiers();
                let is_loop = signals.iter().any(|signal1| {
                    signals.iter().any(|signal2| {
                        all_simple_paths::<Vec<_>, _>(graph, *signal1, *signal2, 0, None)
                            .next()
                            .is_some()
                    })
                });

                // then node call must be inlined
                if is_loop {
                    let called_unitary_node = nodes.get(&node_id).unwrap();

                    // get statements from called node, with corresponding inputs
                    let (retrieved_statements, retrieved_memory) = called_unitary_node
                        .instantiate_statements_and_memory(
                            identifier_creator,
                            inputs,
                            Some(self.pattern),
                            symbol_table,
                        );

                    // remove called node from memory
                    memory.remove_called_node(*node_id);

                    memory.combine(retrieved_memory);
                    retrieved_statements
                } else {
                    // otherwise, just return self
                    vec![self]
                }
            }
            _ => vec![self],
        }
    }
}
