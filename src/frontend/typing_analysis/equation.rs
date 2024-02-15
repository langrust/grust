use crate::error::{Error, TerminationError};
use crate::hir::equation::Equation;
use crate::symbol_table::SymbolTable;

impl Equation {
    /// [Type] the equation.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{equation::Equation, stream_expression::StreamExpression};
    /// use grustine::common::{
    ///     constant::Constant, location::Location, scope::Scope, r#type::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let nodes_context = HashMap::new();
    /// let signals_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// let mut equation = Equation {
    ///     scope: Scope::Local,
    ///     id: String::from("s"),
    ///     signal_type: Type::Integer,
    ///     expression: stream_expression,
    ///     location: Location::default(),
    /// };
    ///
    /// equation.typing(&nodes_context, &signals_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Equation {
            id,
            expression,
            location,
            ..
        } = self;

        expression.typing(symbol_table, errors)?;
        let expression_type = expression.get_type().unwrap();
        let expected_type = symbol_table.get_type(id);
        expression_type.eq_check(expected_type, location.clone(), errors)
    }
}
