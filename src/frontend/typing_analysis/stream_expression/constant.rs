use std::collections::HashMap;

use crate::{
    ast::{stream_expression::StreamExpression, typedef::Typedef},
    common::r#type::Type,
    error::{Error, TerminationError},
};

impl StreamExpression {
    /// Add a [Type] to the constant stream expression.
    pub fn typing_constant(
        &mut self,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // typing a constant stream expression consist of getting the type of the constant
            StreamExpression::Constant {
                constant,
                typing,
                location,
            } => {
                let constant_type = constant.get_type();
                match &constant_type {
                    Type::Enumeration(type_id) => match user_types_context.get(type_id) {
                        Some(Typedef::Enumeration { .. }) => (),
                        _ => {
                            let error = Error::UnknownEnumeration {
                                name: type_id.clone(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            return Err(TerminationError);
                        }
                    },
                    _ => (),
                }
                *typing = Some(constant_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_constant {
    use std::collections::HashMap;

    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{constant::Constant, location::Location};

    #[test]
    fn should_type_constant_stream_expression() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: Some(Constant::Integer(0).get_type()),
            location: Location::default(),
        };

        stream_expression
            .typing_constant(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(stream_expression, control);
    }
}
