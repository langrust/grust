use std::collections::HashMap;

use crate::ast::expression::Expression;
use crate::common::context::Context;
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl Expression {
    /// Add a [Type] to the call expression.
    pub fn typing_call(
        &mut self,
        elements_context: &HashMap<String, Type>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // the type of a call expression in the type of the called element in the context
            Expression::Call {
                id,
                typing,
                location,
            } => {
                let element_type =
                    elements_context.get_element_or_error(id, location.clone(), errors)?;
                *typing = Some(element_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_call {
    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type};
    use crate::error::Error;
    use std::collections::HashMap;

    #[test]
    fn should_type_call_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);

        let mut expression = Expression::Call {
            id: String::from("x"),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Call {
            id: String::from("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing_call(&elements_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_element_call() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);

        let mut expression = Expression::Call {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![Error::UnknownElement {
            name: String::from("y"),
            location: Location::default(),
        }];

        expression
            .typing_call(&elements_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, control);
    }
}
