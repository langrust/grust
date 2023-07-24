use std::collections::HashMap;

use crate::hir::stream_expression::StreamExpression;

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
        context_map: &HashMap<String, Union<String, StreamExpression>>,
    ) {
        match self {
            StreamExpression::Constant { .. } => (),
            StreamExpression::SignalCall { ref mut id, .. } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Union::I1(new_id) => *id = new_id.clone(),
                        Union::I2(new_expression) => *self = new_expression.clone(),
                    }
                }
            }
            StreamExpression::FollowedBy { expression, .. } => {
                expression.replace_by_context(context_map)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::NodeApplication { inputs, .. } => inputs
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::UnitaryNodeApplication { .. } => unreachable!(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .for_each(|(_, expression)| expression.replace_by_context(context_map)),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .for_each(|expression| expression.replace_by_context(context_map)),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                expression.replace_by_context(context_map);
                arms.iter_mut().for_each(|(_, bound, body, expression)| {
                    // todo!("get pattern's context");
                    bound
                        .as_mut()
                        .map(|expression| expression.replace_by_context(context_map));
                    body.iter_mut()
                        .for_each(|equation| equation.expression.replace_by_context(context_map));
                    expression.replace_by_context(context_map);
                })
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                option.replace_by_context(context_map);
                present_body
                    .iter_mut()
                    .for_each(|equation| equation.expression.replace_by_context(context_map));
                present.replace_by_context(context_map);
                default_body
                    .iter_mut()
                    .for_each(|equation| equation.expression.replace_by_context(context_map));
                default.replace_by_context(context_map);
            }
        }
    }
}

#[cfg(test)]
mod replace_by_context {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::{dependencies::Dependencies, stream_expression::StreamExpression};

    #[test]
    fn should_replace_all_occurence_of_identifiers_by_context() {
        let mut expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("y"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0), (String::from("y"), 0)]),
        };

        let context_map = HashMap::from([
            (String::from("x"), Union::I1(String::from("a"))),
            (
                String::from("y"),
                Union::I2(StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("/2"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("b"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                }),
            ),
        ]);

        expression.replace_by_context(&context_map);

        let control = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            inputs: vec![
                StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                },
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("/2"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("b"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("b"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0), (String::from("y"), 0)]),
        };

        assert_eq!(expression, control)
    }
}
