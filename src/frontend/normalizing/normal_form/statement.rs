use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::{
    common::graph::neighbor::Label,
    hir::{
        identifier_creator::IdentifierCreator,
        statement::Statement,
        stream_expression::{StreamExpression, StreamExpressionKind},
    },
    symbol_table::SymbolTable,
};

impl Statement<StreamExpression> {
    /// Change HIR statement into a normal form.
    ///
    /// The normal form of an statement is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normal_form(
        self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<Statement<StreamExpression>> {
        let Statement {
            id,
            mut expression,
            location,
        } = self;

        // change expression into normal form and get additional statements
        let mut statements = match expression.kind {
            StreamExpressionKind::UnitaryNodeApplication {
                node_id,
                ref mut inputs,
                output_id,
            } => {
                let new_statements = inputs
                    .iter_mut()
                    .flat_map(|(_, expression)| {
                        expression.into_signal_call(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect::<Vec<_>>();

                // TODO: change dependencies to be the sum of inputs dependencies?

                new_statements
            }
            _ => expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table),
        };

        // recreate the new statement with modified expression
        let normal_formed_statement = Statement {
            id,
            expression,
            location,
        };

        // push normal_formed statement in the statements storage (in scheduling order)
        statements.push(normal_formed_statement);

        // return statements
        statements
    }
}
