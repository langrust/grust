use crate::hir::stream_expression::StreamExpression;

impl StreamExpression {
    /// Compute dependencies of a signal call.
    pub fn compute_signal_call_dependencies(&self) -> Result<(), ()> {
        match self {
            // signal call depends on called signal with depth of 0
            StreamExpression::SignalCall {
                id, dependencies, ..
            } => {
                dependencies.set(vec![(id.clone(), 0)]);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_signal_call_dependencies {
    use crate::common::{location::Location, r#type::Type};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::stream_expression::StreamExpression;

    #[test]
    fn should_dependencies_of_signal_call_is_signal_with_zero_depth() {
        let stream_expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_signal_call_dependencies()
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
