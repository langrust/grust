mod flow_dependency_graph;
mod inlining;
mod memorize;
mod normal_form;
mod scheduling;

use crate::{hir::file::File, symbol_table::SymbolTable};

impl File {
    /// Normalize HIR nodes in file.
    ///
    /// This is a chain of the following computations:
    /// - unitary nodes generation (check also that all signals are used)
    /// - inlining unitary node calls when needed (shifted causality loops)
    /// - scheduling unitary nodes
    /// - normalizing unitary node application
    /// - memorize node calls and followed by
    ///
    /// # Example
    ///
    /// Let be a node `my_node` and a node `other_node` as follows:
    ///
    /// ```GR
    /// node mem(i: int) {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int, g: int) {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(g-1, v-1).o2;
    ///     out z: int = mem(z).o;
    /// }
    /// ```
    ///
    /// ## Generate unitary nodes
    ///
    /// The generated unitary nodes are the following:
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(v-1).o2;
    /// }
    /// node other_node().z {
    ///     out z: int = mem(z).o;
    /// }
    /// ```
    ///
    /// But `g` is then unused, this will raise an error and stop the compilation.
    ///
    /// ## Inlining unitary nodes
    ///
    /// Suppose that we did not write `g` in the code and that the compilation
    /// succeeded the unitary node generation step. The inlining step will modify
    /// the unitary nodes as follows:
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(v-1).o2;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    ///
    /// In this example, `other_node` calls `mem` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `z` is defined before the input `z`,
    /// which can not be computed by a function call.
    ///
    /// ## Scheduling unitary nodes
    ///
    /// The scheduling step will order the equations of the unitary nodes.
    /// In our example, this will modify the code as bellow.
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     y: int = 1 + my_node(v-1).o2;         // y is before x now
    ///     out x: int = my_node(y, v).o1;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    ///
    /// ## Normal for of unitary nodes
    ///
    /// The last step is the final normal form of the unitary nodes.
    /// The normal form of an unitary node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// This correspond in our example to the following code:
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     x_1: int = v-1;             // x_1 was created
    ///     x_2: int = my_node(x_1).o2; // x_2 was created
    ///     y: int = 1 + x_2;
    ///     out x: int = my_node(y, v).o1;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    pub fn normalize(&mut self, symbol_table: &mut SymbolTable) {
        self.normal_form(symbol_table);
        self.generate_flows_dependency_graphs(symbol_table);
        self.memorize(symbol_table);
        self.inline_when_needed(symbol_table);
        self.schedule()
    }
}
