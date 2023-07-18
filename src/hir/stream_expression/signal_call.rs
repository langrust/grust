use crate::hir::stream_expression::StreamExpression;

impl StreamExpression {
    /// Get dependencies of a signal call.
    pub fn get_signal_call_dependencies(&self) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // signal call depends on called signal with depth of 0
            StreamExpression::SignalCall { id, .. } => Ok(vec![(id.clone(), 0)]),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod get_dependencies_signal_call {
    use crate::common::{location::Location, r#type::Type};
    use crate::hir::stream_expression::StreamExpression;

    #[test]
    fn should_dependencies_of_signal_call_is_signal_with_zero_depth() {
        let stream_expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression.get_signal_call_dependencies().unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
