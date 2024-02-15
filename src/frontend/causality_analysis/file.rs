use crate::{
    error::{Error, TerminationError},
    hir::file::File,
    symbol_table::SymbolTable,
};

impl File {
    /// Check the causality of the file.
    ///
    /// # Example
    /// The folowing file is causal, there is no causality loop.
    /// ```GR
    /// node causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the file that follows is not causal.
    /// In the node `not_causal_node`, signal`o` depends on `x` which depends
    /// on `o`. Values of signals can not be determined, then the compilation
    /// raises a causality error.
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    pub fn causality_analysis(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // check causality for each node
        self.nodes
            .iter()
            .map(|node| node.causal(symbol_table, errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<_, _>>()?;
        // check causality of the optional component
        self.component
            .as_ref()
            .map_or(Ok(()), |component| component.causal(symbol_table, errors))
    }
}
