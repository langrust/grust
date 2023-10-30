use std::collections::HashMap;

use crate::ast::expression::Expression;
use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the map application stream expression.
    pub fn typing_map_application(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // a map application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            StreamExpression::MapApplication {
                function_expression,
                inputs,
                typing,
                location,
            } => {
                // type all inputs
                let test_typing_inputs = inputs
                    .into_iter()
                    .map(|input| {
                        input.typing(
                            nodes_context,
                            signals_context,
                            global_context,
                            user_types_context,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>();

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
                let test_typing_function_expression = function_expression.typing(
                    global_context,
                    &elements_context,
                    user_types_context,
                    errors,
                );

                // test if there were some errors
                test_typing_function_expression?;
                test_typing_inputs?;

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

#[cfg(test)]
mod typing_map_application {
    use crate::ast::{expression::Expression, stream_expression::StreamExpression};
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_map_application_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing_map_application(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_map_application() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Float], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_map_application(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
