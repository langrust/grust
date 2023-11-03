use std::collections::HashMap;

use crate::hir::{equation::Equation, identifier_creator::IdentifierCreator, node::Node};

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
    pub fn inline_when_needed(&mut self, nodes: &HashMap<String, Node>) {
        let mut graph = self.graph.get().unwrap().clone();
        self.unitary_nodes.values_mut().for_each(|unitary_node| {
            // create identifier creator containing the signals
            let mut identifier_creator = IdentifierCreator::from(unitary_node.get_signals());

            // compute new equations for the unitary node
            let mut new_equations: Vec<Equation> = vec![];
            unitary_node.equations.iter().for_each(|equation| {
                let mut retrieved_equations = equation.inline_when_needed_reccursive(
                    &mut identifier_creator,
                    &mut graph,
                    &nodes,
                );
                new_equations.append(&mut retrieved_equations)
            });

            // update node's unitary node
            unitary_node.update_equations(&new_equations)
        })
    }
}
