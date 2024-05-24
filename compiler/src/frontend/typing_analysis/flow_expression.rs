use crate::common::location::Location;
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::flow_expression::{FlowExpression, FlowExpressionKind};
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for FlowExpression {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let location = Location::default();

        match &mut self.kind {
            FlowExpressionKind::Ident { id } => {
                let typing = symbol_table.get_type(*id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            FlowExpressionKind::Sample {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Type::Event(typing) => {
                        // set typing
                        self.typing = Some(Type::Signal(typing.clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectEvent {
                            given_type: given_type.clone(),
                            location: location,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            FlowExpressionKind::Scan {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Type::Signal(typing) => {
                        // set typing
                        self.typing = Some(Type::Event(typing.clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectSignal {
                            given_type: given_type.clone(),
                            location: location,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            FlowExpressionKind::Timeout {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Type::Event(typing) => {
                        // set typing
                        self.typing = Some(Type::Event(todo!("timeout event type")));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectEvent {
                            given_type: given_type.clone(),
                            location: location,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            FlowExpressionKind::Throtle {
                flow_expression,
                delta,
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Type::Signal(typing) => {
                        let delta_ty = delta.get_type();
                        typing.eq_check(&delta_ty, location, errors)?;
                        // set typing
                        self.typing = Some(Type::Signal(typing.clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectSignal {
                            given_type: given_type.clone(),
                            location: location,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            FlowExpressionKind::OnChange { flow_expression } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Type::Signal(typing) => {
                        // set typing
                        self.typing = Some(Type::Event(typing.clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectSignal {
                            given_type: given_type.clone(),
                            location: location,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            FlowExpressionKind::ComponentCall {
                ref component_id,
                ref mut inputs,
                ..
            } => {
                // type all inputs and check their types
                inputs
                    .iter_mut()
                    .map(|(id, input)| {
                        input.typing(symbol_table, errors)?;

                        let input_type = input.get_type().unwrap().convert();
                        let expected_type = symbol_table.get_type(*id);
                        input_type.eq_check(expected_type, self.location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                // get the outputs types fo the called component
                let node_application_type = symbol_table
                    .get_node_outputs(*component_id)
                    .iter()
                    .map(|(_, output_id)| match symbol_table.get_type(*output_id) {
                        Type::Option(ty) => Type::Event(ty.clone()),
                        ty => Type::Signal(Box::new(ty.clone())),
                    })
                    .collect();

                self.typing = Some(Type::Tuple(node_application_type));
                Ok(())
            }
        }
    }

    fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }

    fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
}
