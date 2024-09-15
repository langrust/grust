prelude! {
    hir::{ Dependencies, stream },
    graph::Label,
}

use super::Union;

impl stream::Expr {
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
    pub fn replace_by_context(&mut self, context_map: &HashMap<usize, Union<usize, stream::Expr>>) {
        match self.kind {
            stream::Kind::Expression { ref mut expression } => {
                let option_new_expression =
                    expression.replace_by_context(&mut self.dependencies, context_map);
                if let Some(new_expression) = option_new_expression {
                    *self = new_expression;
                }
            }
            stream::Kind::NodeApplication {
                ref mut memory_id,
                ref mut inputs,
                ..
            } => {
                // replace the id of the called node
                if let Some(element) = context_map.get(&memory_id.unwrap()) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *memory_id = Some(*new_id);
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
            stream::Kind::SomeEvent { ref mut expression } => {
                expression.replace_by_context(context_map);

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::FollowedBy { ref mut id, .. } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *id = *new_id;
                            self.dependencies =
                                Dependencies::from(vec![(*new_id, Label::Weight(1))]);
                        }
                        Union::I2(_) => unreachable!(),
                    }
                }
            }
            stream::Kind::RisingEdge { .. } => unreachable!(),
        }
    }
}
