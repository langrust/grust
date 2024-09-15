prelude! {
    hir::{ Contract, IdentifierCreator, Memory, stream },
}

impl hir::expr::Kind<stream::Expr> {
    /// Increment memory with expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// An expression `0 fby v` increments memory with the buffer
    /// `mem: int = 0 fby v;` and becomes a call to `mem`.
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the
    /// node call `memmy_node_o_: (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        match self {
            Self::Constant { .. }
            | Self::Identifier { .. }
            | Self::Abstraction { .. }
            | Self::Enumeration { .. } => (),
            Self::Unop { expression, .. } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                left_expression.memorize(identifier_creator, memory, contract, symbol_table);
                right_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                true_expression.memorize(identifier_creator, memory, contract, symbol_table);
                false_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Application {
                function_expression,
                inputs,
            } => {
                function_expression.memorize(identifier_creator, memory, contract, symbol_table);
                inputs.iter_mut().for_each(|expression| {
                    expression.memorize(identifier_creator, memory, contract, symbol_table)
                })
            }
            Self::Structure { fields, .. } => fields.iter_mut().for_each(|(_, expression)| {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }),
            Self::Array { elements } | Self::Tuple { elements } => {
                elements.iter_mut().for_each(|expression| {
                    expression.memorize(identifier_creator, memory, contract, symbol_table)
                })
            }
            Self::Match { expression, arms } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                arms.iter_mut().for_each(|(_, option, block, expression)| {
                    option.as_mut().map(|expression| {
                        expression.memorize(identifier_creator, memory, contract, symbol_table)
                    });
                    block.iter_mut().for_each(|statement| {
                        statement.memorize(identifier_creator, memory, contract, symbol_table)
                    });
                    expression.memorize(identifier_creator, memory, contract, symbol_table)
                })
            }
            Self::FieldAccess { expression, .. } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::TupleElementAccess { expression, .. } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Map {
                expression,
                function_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                function_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                initialization_expression.memorize(
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                function_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Sort {
                expression,
                function_expression,
            } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                function_expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            Self::Zip { arrays } => arrays.iter_mut().for_each(|expression| {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }),
        }
    }
}
