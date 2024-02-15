use crate::error::{Error, TerminationError};
use crate::hir::statement::Statement;
use crate::symbol_table::SymbolTable;

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
    /// let mut expression = ExpressionKind::Constant {
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
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Statement {
            id,
            expression,
            location,
        } = self;

        expression.typing(symbol_table, errors)?;
        let expression_type = expression.get_type().unwrap();
        symbol_table.set_type(id, expression_type.clone());
        Ok(())
    }
}
