use std::collections::HashMap;

use crate::ast::stream_expression::StreamExpression;
use crate::common::{context::Context, type_system::Type};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the signal call stream expression.
    pub fn typing_signal_call(
        &mut self,
        signals_context: &HashMap<String, Type>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // the type of a signal call stream expression in the type of the called element in the context
            StreamExpression::SignalCall {
                id,
                typing,
                location,
            } => {
                let element_type =
                    signals_context.get_signal_or_error(id, location.clone(), errors)?;
                *typing = Some(element_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    /// Get dependencies of a signal call.
    pub fn get_dependencies_signal_call(&self) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // signal call depends on called signal with depth of 0
            StreamExpression::SignalCall { id, .. } => Ok(vec![(id.clone(), 0)]),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_call {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{location::Location, type_system::Type};
    use crate::error::Error;
    use std::collections::HashMap;

    #[test]
    fn should_type_call_stream_expression() {
        let mut errors = vec![];
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);

        let mut stream_expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing_signal_call(&signals_context, &mut errors)
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_signal_call() {
        let mut errors = vec![];
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);

        let mut stream_expression = StreamExpression::SignalCall {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![Error::UnknownSignal {
            name: String::from("y"),
            location: Location::default(),
        }];

        stream_expression
            .typing_signal_call(&signals_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, control);
    }
}

#[cfg(test)]
mod get_dependencies_signal_call {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::location::Location;

    #[test]
    fn should_dependencies_of_signal_call_is_signal_with_zero_depth() {
        let stream_expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: None,
            location: Location::default(),
        };

        let dependencies = stream_expression.get_dependencies_signal_call().unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
