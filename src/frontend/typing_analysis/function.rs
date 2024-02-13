use std::collections::HashMap;

use crate::error::{Error, TerminationError};
use crate::hir::{function::Function, typedef::Typedef};
use crate::symbol_table::{SymbolKind, SymbolTable};

impl Function {
    /// [Type] the function.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     expression::Expression, function::Function, statement::Statement,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, r#type::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         Statement {
    ///             id: String::from("x"),
    ///             element_type: Type::Integer,
    ///             expression: Expression::Call {
    ///                 id: String::from("i"),
    ///                 typing: None,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         }
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         Expression::Call {
    ///             id: String::from("x"),
    ///             typing: None,
    ///             location: Location::default(),
    ///         }
    ///     ),
    ///     location: Location::default(),
    /// };
    ///
    /// function.typing(&global_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Function {
            id,
            inputs,
            statements,
            returned,
            location,
            ..
        } = self;

        // type all statements
        statements
            .iter_mut()
            .map(|statement| statement.typing(symbol_table, user_types_context, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // type returned expression
        returned.typing(symbol_table, user_types_context, errors)?;

        // check returned type
        let symbol = symbol_table
            .get_symbol(id)
            .expect("there should be a symbol");
        match symbol.kind() {
            SymbolKind::Function { output_typing, .. } => {
                returned
                    .get_type()
                    .unwrap()
                    .eq_check(output_typing, location.clone(), errors)
            }
            _ => unreachable!(),
        }
    }
}
