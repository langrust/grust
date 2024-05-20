use std::collections::HashMap;

use crate::{
    common::label::Label,
    hir::{
        dependencies::Dependencies, expression::ExpressionKind, stream_expression::StreamExpression,
    },
};

use super::Union;

impl ExpressionKind<StreamExpression> {
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
        dependencies: &mut Dependencies,
        context_map: &HashMap<usize, Union<usize, StreamExpression>>,
    ) -> Option<StreamExpression> {
        match self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Abstraction { .. }
            | ExpressionKind::Enumeration { .. } => None,
            ExpressionKind::Identifier { ref mut id } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Union::I1(new_id) => {
                            *id = *new_id;
                            *dependencies = Dependencies::from(vec![(*new_id, Label::Weight(0))]);
                            None
                        }
                        Union::I2(new_expression) => Some(new_expression.clone()),
                    }
                } else {
                    None
                }
            }
            ExpressionKind::Unop { expression, .. } => {
                expression.replace_by_context(context_map);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
                None
            }
            ExpressionKind::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                left_expression.replace_by_context(context_map);
                right_expression.replace_by_context(context_map);

                let mut expression_dependencies = left_expression.get_dependencies().clone();
                let mut other_dependencies = right_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                expression.replace_by_context(context_map);
                true_expression.replace_by_context(context_map);
                false_expression.replace_by_context(context_map);

                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut other_dependencies = true_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);
                let mut other_dependencies = false_expression.get_dependencies().clone();
                expression_dependencies.append(&mut other_dependencies);

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Application { ref mut inputs, .. } => {
                inputs
                    .iter_mut()
                    .for_each(|expression| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            ExpressionKind::Structure { ref mut fields, .. } => {
                fields
                    .iter_mut()
                    .for_each(|(_, expression)| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            ExpressionKind::Array { ref mut elements }
            | ExpressionKind::Tuple { ref mut elements } => {
                elements
                    .iter_mut()
                    .for_each(|expression| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            ExpressionKind::Match {
                ref mut expression,
                ref mut arms,
                ..
            } => {
                expression.replace_by_context(context_map);
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(pattern, bound, body, matched_expression)| {
                        let local_signals = pattern.identifiers();

                        // remove identifiers created by the pattern from the context
                        let context_map = context_map
                            .clone()
                            .into_iter()
                            .filter(|(key, _)| !local_signals.contains(key))
                            .collect();

                        if let Some(expression) = bound.as_mut() {
                            expression.replace_by_context(&context_map);
                            let mut bound_dependencies = expression
                                .get_dependencies()
                                .clone()
                                .into_iter()
                                .filter(|(signal, _)| !local_signals.contains(signal))
                                .collect();
                            expression_dependencies.append(&mut bound_dependencies);
                        };

                        body.iter_mut().for_each(|statement| {
                            statement.expression.replace_by_context(&context_map)
                        });

                        matched_expression.replace_by_context(&context_map);
                        let mut matched_expression_dependencies = matched_expression
                            .get_dependencies()
                            .clone()
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect::<Vec<(usize, Label)>>();
                        expression_dependencies.append(&mut matched_expression_dependencies);
                    });

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::When {
                ref mut option,
                ref mut present_body,
                ref mut present,
                ref mut default_body,
                ref mut default,
                ..
            } => {
                option.replace_by_context(context_map);
                let mut option_dependencies = option.get_dependencies().clone();

                debug_assert!(present_body.is_empty());
                // present_body
                //     .iter_mut()
                //     .for_each(|statements| statements.expression.replace_by_context(context_map));

                present.replace_by_context(context_map);
                let mut present_dependencies = present.get_dependencies().clone();

                debug_assert!(default_body.is_empty());
                // default_body
                //     .iter_mut()
                //     .for_each(|statements| statements.expression.replace_by_context(context_map));

                default.replace_by_context(context_map);
                let mut default_dependencies = default.get_dependencies().clone();

                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);
                *dependencies = Dependencies::from(option_dependencies);
                None
            }
            ExpressionKind::FieldAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::TupleElementAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Map {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Fold {
                ref mut expression,
                ref mut initialization_expression,
                ..
            } => {
                expression.replace_by_context(context_map);
                initialization_expression.replace_by_context(context_map);
                // get matched expressions dependencies
                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut initialization_expression_dependencies =
                    expression.get_dependencies().clone();
                expression_dependencies.append(&mut initialization_expression_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Sort {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Zip { ref mut arrays, .. } => {
                arrays
                    .iter_mut()
                    .for_each(|array| array.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    arrays
                        .iter()
                        .flat_map(|array| array.get_dependencies().clone())
                        .collect(),
                );
                None
            }
        }
    }
}
