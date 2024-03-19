use std::collections::HashMap;

use crate::hir::{
    dependencies::Dependencies,
    expression::ExpressionKind,
    stream_expression::{StreamExpression, StreamExpressionKind},
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
        context_map: &HashMap<usize, Union<usize, StreamExpression>>,
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
}
