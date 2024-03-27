use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::interface::{FlowExpression, FlowExpressionKind, Interface};
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for Interface {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // type all statements
        self.flow_statements
            .iter_mut()
            .map(|statement| statement.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()
    }
}

impl TypeAnalysis for FlowExpression {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match &mut self.kind {
            FlowExpressionKind::Ident { id } => {
                let typing = symbol_table.get_type(*id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            FlowExpressionKind::Timeout {
                flow_expression, ..
            } => {
                flow_expression.typing(symbol_table, errors)?;
                // get expression type
                let typing = flow_expression.get_type().unwrap();
                // set typing
                self.typing = Some(typing.clone());
                Ok(())
            }
            FlowExpressionKind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => {
                flow_expression_1.typing(symbol_table, errors)?;
                flow_expression_2.typing(symbol_table, errors)?;
                // get expressions type
                let typing_1 = flow_expression_1.get_type().unwrap();
                let typing_2 = flow_expression_2.get_type().unwrap();
                // check equality
                typing_1.eq_check(typing_2, self.location.clone(), errors)?;
                // set typing
                self.typing = Some(typing_1.clone());
                Ok(())
            }
            FlowExpressionKind::Zip {
                flow_expression_1,
                flow_expression_2,
            } => {
                flow_expression_1.typing(symbol_table, errors)?;
                flow_expression_2.typing(symbol_table, errors)?;
                // get expressions type
                let typing_1 = flow_expression_1.get_type().unwrap();
                let typing_2 = flow_expression_2.get_type().unwrap();
                // set typing
                self.typing = Some(Type::Tuple(vec![typing_1.clone(), typing_2.clone()]));
                Ok(())
            }
            FlowExpressionKind::ComponentCall {
                ref mut inputs,
                ref signal_id,
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

                // get the called signal type
                let node_application_type = symbol_table.get_type(*signal_id);

                self.typing = Some(Type::Signal(Box::new(node_application_type.clone())));
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
