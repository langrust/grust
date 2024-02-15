use crate::error::{Error, TerminationError};
use crate::hir::file::File;
use crate::symbol_table::SymbolTable;

impl File {
    /// [Type] the entire file.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{
    ///     equation::Equation, expression::Expression, function::Function,
    ///     file::File, node::Node, statement::Statement, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{location::Location, scope::Scope, r#type::Type};
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i"),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let function = Function {
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
    /// let mut file = File {
    ///     typedefs: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    ///
    /// file.typing(&mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let File {
            typedefs,
            functions,
            nodes,
            component,
            ..
        } = self;

        // typing nodes
        nodes
            .iter_mut()
            .map(|node| node.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // typing component
        component
            .as_mut()
            .map_or(Ok(()), |component| component.typing(symbol_table, errors))?;

        // typing functions
        functions
            .iter_mut()
            .map(|function| function.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()
    }
}
