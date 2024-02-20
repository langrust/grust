use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::function::Function;
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for Function {
    /// [Type] the function.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::BTreeMap;
    ///
    /// use grustine::ast::{
    ///     expression::Expression, function::Function, statement::Statement,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, r#type::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let global_context = BTreeMap::new();
    /// let user_types_context = BTreeMap::new();
    ///
    /// let mut function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         Statement {
    ///             id: String::from("x"),
    ///             element_type: Type::Integer,
    ///             expression: ExpressionKind::Identifier {
    ///                 id: String::from("i"),
    ///                 typing: None,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         }
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         ExpressionKind::Identifier {
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
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Function {
            id,
            statements,
            returned,
            location,
            ..
        } = self;

        // type all statements
        statements
            .iter_mut()
            .map(|statement| statement.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // type returned expression
        returned.typing(symbol_table, errors)?;

        // check returned type
        let expected_type = symbol_table.get_function_output_type(id);
        returned
            .get_type()
            .unwrap()
            .eq_check(expected_type, location.clone(), errors)
    }
}
