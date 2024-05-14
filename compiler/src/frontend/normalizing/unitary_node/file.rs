use crate::error::{Error, TerminationError};
use crate::hir::file::File;
use crate::symbol_table::SymbolTable;

impl File {
    /// Generate unitary nodes.
    ///
    /// It also changes node application expressions into unitary node application
    /// and removes unused inputs from those unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` and a node `other_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int, g: int) {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = my_node(g-1, v).o2;
    /// }
    /// ```
    ///
    /// The generated unitary nodes are the following:
    ///
    /// ```GR
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int) {           // g is then unused and will raise an error
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = my_node(v).o2;
    /// }
    /// ```
    pub fn generate_unitary_nodes(
        &mut self,
        symbol_table: &mut SymbolTable,
        creusot_contract: bool,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // unitary nodes computations, it induces unused signals tracking
        self.nodes
            .iter_mut()
            .map(|node| node.generate_unitary_nodes(symbol_table, creusot_contract, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // change node application to unitary node application
        self.nodes.iter_mut().for_each(|node| {
            node.change_node_application_into_unitary_node_application(symbol_table)
        });

        // change component application to unitary node application
        self.interface.iter_mut().for_each(|statement| {
            statement
                .expression
                .change_node_application_into_unitary_node_application(symbol_table)
        });

        // Debug: test there is no NodeApplication
        debug_assert!(self.no_node_application());

        Ok(())
    }
}
