use std::collections::HashMap;

use crate::ast::{
    stream_expression::StreamExpression, type_system::Type, user_defined_type::UserDefinedType,
};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the array stream expression.
    pub fn typing_array(
        &mut self,
        signals_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            StreamExpression::Array {
                elements,
                typing,
                location,
            } => {
                elements
                    .into_iter()
                    .map(|element| element.typing(signals_context, user_types_context, errors))
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                let first_type = elements[0].get_type().unwrap();
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                let array_type = Type::Array(Box::new(first_type.clone()), elements.len());

                *typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_array {
    use crate::ast::{
        constant::Constant, location::Location, stream_expression::StreamExpression,
        type_system::Type,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_array_stream_expression() {
        let mut errors = vec![];
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Array(Box::new(Type::Integer), 2)),
            location: Location::default(),
        };

        stream_expression
            .typing_array(&signals_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_multiple_types_array() {
        let mut errors = vec![];
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Float(1.0),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };

        let error = stream_expression
            .typing_array(&signals_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }
}
