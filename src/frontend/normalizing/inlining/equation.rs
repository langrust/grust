use std::collections::HashMap;

use crate::hir::{
    equation::Equation, identifier_creator::IdentifierCreator, stream_expression::StreamExpression,
};

use super::Union;

impl Equation {
    /// Add the equation identifier to the identifier creator.
    ///
    /// It will add the equation identifier to the identifier creator.
    /// If the identifier already exists, then the new identifer created by
    /// the identifier creator will be added to the renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<String, Union<String, StreamExpression>>,
    ) {
        let new_id =
            identifier_creator.new_identifier(String::new(), self.id.clone(), String::new());
        if new_id.ne(&self.id) {
            assert!(context_map
                .insert(self.id.clone(), Union::I1(new_id))
                .is_none());
        }
    }

    /// Replace identifier occurence by element in context.
    ///
    /// It will return a new equation where the expression has been modified
    /// according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function
    /// with the equation `z = x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<String, Union<String, StreamExpression>>,
    ) -> Equation {
        let mut new_equation = self.clone();
        if let Some(element) = context_map.get(&new_equation.id) {
            match element {
                Union::I1(new_id) | Union::I2(StreamExpression::SignalCall { id: new_id, .. }) => {
                    new_equation.id = new_id.clone()
                }
                Union::I2(_) => unreachable!(),
            }
        }
        new_equation.expression.replace_by_context(context_map);
        new_equation
    }
}

#[cfg(test)]
mod add_necessary_renaming {
    use std::collections::HashMap;

    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::identifier_creator::IdentifierCreator;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_the_equation_id_to_the_identifier_creator_if_id_is_not_used() {
        let equation = Equation {
            id: String::from("y"),
            scope: Scope::Local,
            expression: StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };

        let mut context_map = HashMap::from([(String::from("x"), Union::I1(String::from("a")))]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        equation.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = IdentifierCreator::from(vec![String::from("a"), String::from("y")]);

        assert_eq!(identifier_creator, control)
    }

    #[test]
    fn should_add_the_equation_id_to_the_context_if_id_is_already_used() {
        let equation = Equation {
            id: String::from("a"),
            scope: Scope::Local,
            expression: StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };

        let mut context_map = HashMap::from([(String::from("x"), Union::I1(String::from("a")))]);
        let mut identifier_creator = IdentifierCreator::from(vec![String::from("a")]);

        equation.add_necessary_renaming(&mut identifier_creator, &mut context_map);

        let control = HashMap::from([
            (String::from("x"), Union::I1(String::from("a"))),
            (String::from("a"), Union::I1(String::from("a_1"))),
        ]);
        assert_eq!(context_map, control)
    }
}

#[cfg(test)]
mod replace_by_context {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, stream_expression::StreamExpression,
    };

    #[test]
    fn should_replace_all_occurence_of_identifiers_by_context() {
        let equation = Equation {
            id: String::from("z"),
            scope: Scope::Local,
            expression: StreamExpression::MapApplication {
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
                dependencies: Dependencies::from(vec![
                    (String::from("x"), 0),
                    (String::from("y"), 0),
                ]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
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
            (String::from("z"), Union::I1(String::from("c"))),
        ]);

        let replaced_equation = equation.replace_by_context(&context_map);

        let control = Equation {
            id: String::from("c"),
            scope: Scope::Local,
            expression: StreamExpression::MapApplication {
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
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
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
                dependencies: Dependencies::from(vec![
                    (String::from("x"), 0),
                    (String::from("y"), 0),
                ]),
            },
            signal_type: Type::Integer,
            location: Location::default(),
        };

        assert_eq!(replaced_equation, control)
    }
}
