use std::collections::HashMap;

use crate::hir::{dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression};

use super::Union;

impl Dependencies {
    /// Replace identifier occurence by dependencies of element in context.
    ///
    /// It will modify the dependencies according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the dependencies of the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` which depends
    /// on `x` and `y` will depends on `a` and `b`.
    pub fn replace_by_context(
        &mut self,
        context_map: &HashMap<String, Union<Signal, StreamExpression>>,
    ) {
        let new_dependencies = self
            .get()
            .unwrap()
            .iter()
            .flat_map(|(id, depth)| match context_map.get(id) {
                Some(Union::I1(Signal { id: new_id, .. })) => vec![(new_id.clone(), *depth)],
                Some(Union::I2(expression)) => expression
                    .get_dependencies()
                    .iter()
                    .map(|(new_id, new_depth)| (new_id.clone(), depth + new_depth))
                    .collect(),
                None => vec![],
            })
            .collect();

        *self = Dependencies::from(new_dependencies);
    }
}

#[cfg(test)]
mod replace_by_context {
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::normalizing::inlining::Union;
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_replace_all_occurence_of_identifiers_by_context() {
        let mut dependencies =
            Dependencies::from(vec![(String::from("x"), 0), (String::from("y"), 0)]);

        let context_map = HashMap::from([
            (
                String::from("x"),
                Union::I1(Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                }),
            ),
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
                        scope: Scope::Local,
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

        dependencies.replace_by_context(&context_map);

        let control = Dependencies::from(vec![(String::from("a"), 0), (String::from("b"), 0)]);

        assert_eq!(dependencies, control)
    }
}
