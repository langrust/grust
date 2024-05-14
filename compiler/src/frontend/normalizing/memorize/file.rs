use crate::hir::file::File;
use crate::symbol_table::SymbolTable;

impl File {
    /// Create memory for HIR nodes' unitary nodes.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         memmy_node_o_: (my_node, o);
    ///     },
    /// }
    /// ```
    ///
    /// This example is tested in source.
    pub fn memorize(&mut self, symbol_table: &mut SymbolTable) {
        self.nodes
            .iter_mut()
            .for_each(|node| node.memorize(symbol_table));

        // Debug: test there is no FollowedBy expressions
        debug_assert!(self.no_fby());
    }
}
