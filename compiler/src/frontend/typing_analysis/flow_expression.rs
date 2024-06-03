prelude! {
    frontend::TypeAnalysis,
    hir::flow,
}

impl TypeAnalysis for flow::Expr {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let location = Location::default();

        match &mut self.kind {
            flow::Kind::Ident { id } => {
                let typing = symbol_table.get_type(*id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            flow::Kind::Sample {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Typ::Event(typing) => {
                        // set typing
                        self.typing = Some(Typ::signal((**typing).clone()));
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
            flow::Kind::Scan {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Typ::Signal(typing) => {
                        // set typing
                        self.typing = Some(Typ::event((**typing).clone()));
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
            flow::Kind::Timeout {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                match flow_expression.get_type().unwrap() {
                    Typ::Event(typing) => {
                        // set typing
                        self.typing = Some(Typ::event(Typ::timeout((**typing).clone())));
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
            flow::Kind::Throtle {
                flow_expression,
                delta,
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Typ::Signal(typing) => {
                        let delta_ty = delta.get_type();
                        typing.eq_check(&delta_ty, location, errors)?;
                        // set typing
                        self.typing = Some(Typ::signal((**typing).clone()));
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
            flow::Kind::OnChange { flow_expression } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                match typing {
                    Typ::Signal(typing) => {
                        // set typing
                        self.typing = Some(Typ::event((**typing).clone()));
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
            flow::Kind::ComponentCall {
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
                    .collect::<TRes<()>>()?;

                // get the outputs types of the called component
                let mut outputs_types = symbol_table
                    .get_node_outputs(*component_id)
                    .iter()
                    .map(|(_, output_id)| match symbol_table.get_type(*output_id) {
                        Typ::SMTimeout(ty) => Typ::event(Typ::timeout((**ty).clone())),
                        Typ::SMEvent(ty) => Typ::event((**ty).clone()),
                        ty => Typ::signal(ty.clone()),
                    })
                    .collect::<Vec<_>>();

                // construct node application type
                let node_application_type = if outputs_types.len() == 1 {
                    outputs_types.pop().unwrap()
                } else {
                    Typ::tuple(outputs_types)
                };

                self.typing = Some(node_application_type);
                Ok(())
            }
        }
    }

    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
}
