use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::stream_expression::{StreamExpression, StreamExpressionKind};
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for StreamExpression {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            StreamExpressionKind::FollowedBy {
                ref mut constant,
                ref mut expression,
            } => {
                // type expressions
                constant.typing(symbol_table, errors)?;
                expression.typing(symbol_table, errors)?;

                // check it is equal to constant type
                let expression_type = expression.get_type().unwrap();
                let constant_type = constant.get_type().unwrap();
                expression_type.eq_check(constant_type, self.location.clone(), errors)?;

                self.typing = Some(constant_type.clone());
                Ok(())
            }

            StreamExpressionKind::NodeApplication {
                node_id,
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
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                // get the called signal type
                let node_application_type = {
                    let mut outputs_types = symbol_table
                        .get_node_outputs(node_id)
                        .iter()
                        .map(|(_, output_signal)| symbol_table.get_type(*output_signal).clone())
                        .collect::<Vec<_>>();
                    if outputs_types.len() == 1 {
                        outputs_types.pop().unwrap()
                    } else {
                        Type::Tuple(outputs_types)
                    }
                };

                self.typing = Some(node_application_type);
                Ok(())
            }

            StreamExpressionKind::Expression { ref mut expression } => {
                self.typing = Some(expression.typing(&self.location, symbol_table, errors)?);
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
