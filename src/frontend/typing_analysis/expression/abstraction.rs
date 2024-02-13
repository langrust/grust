use std::collections::HashMap;

use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression, typedef::Typedef};
use crate::symbol_table::{SymbolKind, SymbolTable};

impl Expression {
    /// Add a [Type] to the abstraction expression.
    pub fn typing_abstraction(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // the type of a typed abstraction is computed by adding inputs to
            // the context and typing the function body expression
            Expression::Abstraction {
                inputs,
                expression,
                typing,
                location,
            } => {
                // type the abstracted expression with the local context
                expression.typing(symbol_table, user_types_context, errors)?;

                // compute abstraction type
                let input_types = inputs
                    .iter()
                    .map(|id| {
                        let symbol = symbol_table
                            .get_symbol(id)
                            .expect("there should be a symbol");
                        match symbol.kind() {
                            SymbolKind::Identifier { typing } => typing.clone(),
                            _ => unreachable!(),
                        }
                    })
                    .collect::<Vec<_>>();
                let abstraction_type = Type::Abstract(
                    input_types,
                    Box::new(expression.get_type().unwrap().clone()),
                );

                *typing = Some(abstraction_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
