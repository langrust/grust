use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::node::Node;
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for Node {
    /// [Type] the node.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::BTreeMap;
    ///
    /// use grustine::ast::{
    ///     equation::Equation, node::Node, node_description::NodeDescription,
    ///     stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, scope::Scope, r#type::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut nodes_context = BTreeMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     NodeDescription {
    ///         is_component: false,
    ///         inputs: vec![(String::from("i"), Type::Integer)],
    ///         outputs: BTreeMap::from([(String::from("o"), Type::Integer)]),
    ///         locals: BTreeMap::from([(String::from("x"), Type::Integer)]),
    ///     }
    /// );
    /// let global_context = BTreeMap::new();
    /// let user_types_context = BTreeMap::new();
    ///
    /// let mut node =Node { assertions: Default::default(), contract: Default::default(),
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
    /// node.typing(&nodes_context, &global_context, &user_types_context, &mut errors).unwrap();
    /// ```
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            unscheduled_equations,
            ..
        } = self;

        // type all equations
        unscheduled_equations
            .iter_mut()
            .map(|(_, equation)| equation.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()
    }
}
