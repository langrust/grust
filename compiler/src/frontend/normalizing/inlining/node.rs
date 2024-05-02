use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    hir::{node::Node, unitary_node::UnitaryNode},
    symbol_table::SymbolTable,
};

impl Node {
    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// node fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed(
        &mut self,
        unitary_nodes: &HashMap<usize, UnitaryNode>,
        symbol_table: &mut SymbolTable,
    ) {
        self.unitary_nodes
            .values_mut()
            .sorted_by_key(|unitary_node| unitary_node.id)
            .for_each(|unitary_node| unitary_node.inline_when_needed(unitary_nodes, symbol_table))
    }
}
