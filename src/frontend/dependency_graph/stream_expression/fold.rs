use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a fold stream expression.
    pub fn compute_fold_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of fold are dependencies of the folded expression
            StreamExpression::Fold {
                expression,
                initialization_expression,
                dependencies,
                ..
            } => {
                // get folded expression dependencies
                expression.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                // get initialization expression dependencies
                initialization_expression.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut initialization_expression_dependencies =
                    initialization_expression.get_dependencies().clone();

                expression_dependencies.append(&mut initialization_expression_dependencies);
                // push in fold dependencies
                dependencies.set(expression_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_fold_dependencies {
    use crate::ast::expression::Expression;
    use crate::common::constant::Constant;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_fold() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                },
                typing: Type::Array(Box::new(Type::Integer), 3),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_fold_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let mut dependencies = stream_expression.get_dependencies().clone();
        dependencies.sort_unstable();

        let control = vec![(String::from("a"), 0)];

        assert_eq!(dependencies, control)
    }
}
