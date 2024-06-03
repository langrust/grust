prelude! {
    hir::{ Contract, IdentifierCreator, Memory, Stmt, stream },
}

impl Stmt<stream::Expr> {
    /// Increment memory with statement's expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// An statement `x: int = 0 fby v;` increments memory with the buffer
    /// `mem: int = 0 fby v;` and becomes `x: int = mem;`.
    ///
    /// An statement `x: int = my_node(s, x_1).o;` increments memory with the
    /// node call `memmy_node_o_: (my_node, o);` and the statement is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        self.expression
            .memorize(identifier_creator, memory, contract, symbol_table)
    }
}
