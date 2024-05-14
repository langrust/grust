use std::collections::HashMap;

use crate::{hir::file::File, symbol_table::SymbolTable};

impl File {
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
    pub fn inline_when_needed(&mut self, symbol_table: &mut SymbolTable) {
        let nodes = self
            .nodes
            .iter()
            .flat_map(|node| {
                node.unitary_nodes
                    .values()
                    .map(|unitary_node| (unitary_node.id.clone(), unitary_node.clone()))
            })
            .collect::<HashMap<_, _>>();
        self.nodes
            .iter_mut()
            .for_each(|node| node.inline_when_needed(&nodes, symbol_table))
    }
}
