use std::collections::HashMap;

use crate::error::{Error, TerminationError};
use crate::hir::{statement::Statement, typedef::Typedef};
use crate::symbol_table::{SymbolKind, SymbolTable};

impl Statement {
    /// [Type] the statement.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     expression::Expression, statement::Statement,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, r#type::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// let mut statement = Statement {
    ///     id: String::from("x"),
    ///     element_type: Type::Integer,
    ///     expression: expression,
    ///     location: Location::default(),
    /// };
    ///
    /// statement.typing(&global_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Statement {
            id,
            expression,
            location,
        } = self;

        expression.typing(symbol_table, user_types_context, errors)?;
        let expression_type = expression.get_type().unwrap();

        let symbol = symbol_table
            .get_symbol(id)
            .expect("there should be a symbol");
        match symbol.kind() {
            SymbolKind::Identifier { typing } => {
                expression_type.eq_check(typing, location.clone(), errors)
            }
            _ => unreachable!(),
        }
    }
}
