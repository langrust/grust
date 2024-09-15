//! LanGRust [stream::Expr] typing analysis module.

prelude! {
    frontend::TypeAnalysis,
    hir::stream,
}

impl TypeAnalysis for stream::Expr {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match self.kind {
            stream::Kind::FollowedBy {
                id,
                ref mut constant,
            } => {
                // type expressions
                constant.typing(symbol_table, errors)?;

                // check it is equal to constant type
                let id_type = symbol_table.get_type(id);
                let constant_type = constant.get_type().unwrap();
                id_type.eq_check(constant_type, self.location.clone(), errors)?;

                // check the scope is not 'very_local'
                if let Scope::VeryLocal = symbol_table.get_scope(id) {
                    return Err(TerminationError);// todo generate error
                }

                self.typing = Some(constant_type.clone());
                Ok(())
            }

            stream::Kind::NodeApplication {
                called_node_id,
                ref mut inputs,
                ..
            } => {
                // type all inputs and check their types
                inputs
                    .iter_mut()
                    .map(|(id, input)| {
                        input.typing(symbol_table, errors)?;

                        let input_type = input.typing.as_ref().unwrap();
                        let expected_type = symbol_table.get_type(*id);
                        input_type.eq_check(expected_type, self.location.clone(), errors)
                    })
                    .collect::<TRes<()>>()?;

                // get the called signal type
                let node_application_type = {
                    let mut outputs_types = symbol_table
                        .get_node_outputs(called_node_id)
                        .iter()
                        .map(|(_, output_signal)| symbol_table.get_type(*output_signal).clone())
                        .collect::<Vec<_>>();
                    if outputs_types.len() == 1 {
                        outputs_types.pop().unwrap()
                    } else {
                        Typ::tuple(outputs_types)
                    }
                };

                self.typing = Some(node_application_type);
                Ok(())
            }

            stream::Kind::Expression { ref mut expression } => {
                self.typing = Some(expression.typing(&self.location, symbol_table, errors)?);
                Ok(())
            }

            stream::Kind::SomeEvent { ref mut expression } => {
                expression.typing(symbol_table, errors)?;
                let expression_type = expression.get_type().unwrap().clone();
                self.typing = Some(Typ::sm_event(expression_type));
                Ok(())
            }

            stream::Kind::NoneEvent => {
                self.typing = Some(Typ::sm_event(Typ::Any));
                Ok(())
            }
            stream::Kind::RisingEdge { ref mut expression } => {
                expression.typing(symbol_table, errors)?;
                // check expr is a boolean
                let expr_type = expression.get_type().unwrap().clone();
                let expected = Typ::bool();
                expr_type.eq_check(&expected, self.location.clone(), errors)?;
                // set the type
                self.typing = Some(expected);
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
