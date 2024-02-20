use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::{
    common::graph::neighbor::Label,
    hir::{
        dependencies::Dependencies,
        expression::ExpressionKind,
        identifier_creator::IdentifierCreator,
        memory::Memory,
        statement::Statement,
        stream_expression::{StreamExpression, StreamExpressionKind},
        unitary_node::UnitaryNode,
    },
    symbol_table::SymbolTable,
};

use super::Union;

impl StreamExpression {
    /// Replace identifier occurence by element in context.
    ///
    /// It will modify the expression according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` will become
    /// `a + b/2`.
    pub fn replace_by_context(
        &mut self,
        context_map: &BTreeMap<usize, Union<usize, StreamExpression>>,
    ) {
        match self.kind {
            StreamExpressionKind::Expression { ref mut expression } => {
                let option_new_expression =
                    expression.replace_by_context(&mut self.dependencies, context_map);
                if let Some(new_expression) = option_new_expression {
                    *self = new_expression;
                }
            }
            StreamExpressionKind::UnitaryNodeApplication {
                ref mut node_id,
                ref mut inputs,
                ..
            } => {
                // replace the id of the called node
                if let Some(element) = context_map.get(node_id) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(StreamExpression {
                            kind:
                                StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *node_id = new_id.clone();
                        }
                        Union::I2(_) => unreachable!(),
                    }
                }

                inputs
                    .iter_mut()
                    .for_each(|(_, expression)| expression.replace_by_context(context_map));

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            StreamExpressionKind::FollowedBy { .. }
            | StreamExpressionKind::NodeApplication { .. } => unreachable!(),
        }
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// ```
    /// In this example, if an expression `semi_fib(fib).o` is assigned to the
    /// signal `fib` there is no causality loop.
    /// But we need to inline the code, a function can not compute an output
    /// before knowing the input.
    pub fn inline_when_needed(
        &mut self,
        signal_id: usize,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        graph: &DiGraphMap<usize, Label>,
        symbol_table: &mut SymbolTable,
        unitary_nodes: &BTreeMap<usize, UnitaryNode>,
    ) -> Vec<Statement<StreamExpression>> {
        match &mut self.kind {
            StreamExpressionKind::UnitaryNodeApplication { .. } => unreachable!(),
            StreamExpressionKind::FollowedBy { .. } => unreachable!(),
            StreamExpressionKind::NodeApplication { .. } => unreachable!(),
            StreamExpressionKind::Expression { expression } => expression.inline_when_needed(
                signal_id,
                memory,
                identifier_creator,
                graph,
                symbol_table,
                unitary_nodes,
            ),
        }
    }
}
