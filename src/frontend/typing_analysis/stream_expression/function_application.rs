use std::collections::HashMap;

use crate::ast::expression::Expression;
use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the function application stream expression.
    pub fn typing_function_application(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // a function application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            StreamExpression::FunctionApplication {
                function_expression,
                inputs,
                typing,
                location,
            } => {
                // type all inputs
                inputs
                    .iter_mut()
                    .map(|input| {
                        input.typing(
                            nodes_context,
                            signals_context,
                            global_context,
                            user_types_context,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let input_types = inputs
                    .iter()
                    .map(|input| input.get_type().unwrap().clone())
                    .collect::<Vec<_>>();

                if let Expression::Abstraction {
                    inputs: abstraction_inputs,
                    expression,
                    typing,
                    location,
                } = function_expression
                {
                    // transform abstraction in typed abstraction
                    let typed_inputs = abstraction_inputs
                        .clone()
                        .into_iter()
                        .zip(input_types.clone())
                        .collect::<Vec<_>>();
                    *function_expression = Expression::TypedAbstraction {
                        inputs: typed_inputs,
                        expression: expression.clone(),
                        typing: typing.clone(),
                        location: location.clone(),
                    };
                };

                // type the function expression
                let elements_context = global_context.clone();
                function_expression.typing(
                    global_context,
                    &elements_context,
                    user_types_context,
                    errors,
                )?;

                // compute the application type
                let application_type = function_expression.get_type_mut().unwrap().apply(
                    input_types,
                    location.clone(),
                    errors,
                )?;

                *typing = Some(application_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
