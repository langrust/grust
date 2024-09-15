prelude! {
    graph::Label,
    hir::{ Dependencies, stream, },
}

use super::Union;

impl hir::expr::Kind<stream::Expr> {
    /// Replace identifier occurrence by element in context.
    ///
    /// It will modify the expression according to the context:
    ///
    /// - if an identifier is mapped to another identifier, then rename all occurrence of the
    ///   identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to the identifier by
    ///   the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` will become `a + b/2`.
    pub fn replace_by_context(
        &mut self,
        dependencies: &mut Dependencies,
        context_map: &HashMap<usize, Union<usize, stream::Expr>>,
    ) -> Option<stream::Expr> {
        match self {
            Self::Constant { .. } | Self::Abstraction { .. } | Self::Enumeration { .. } => None,
            Self::Identifier { ref mut id } => {
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
            Self::Unop { expression, .. } => {
                expression.replace_by_context(context_map);
                *dependencies = Dependencies::from(expression.get_dependencies().clone());
                None
            }
            Self::Binop {
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
            Self::IfThenElse {
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
            Self::Application { ref mut inputs, .. } => {
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
            Self::Structure { ref mut fields, .. } => {
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
            Self::Array { ref mut elements } | Self::Tuple { ref mut elements } => {
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
            Self::Match {
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
            Self::FieldAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::TupleElementAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Map {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Fold {
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
            Self::Sort {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            Self::Zip { ref mut arrays, .. } => {
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
