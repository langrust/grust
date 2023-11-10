use crate::{
    common::r#type::Type,
    lir::{expression::Expression, statement::Statement},
};

/// A step function.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Step {
    /// The node's name.
    pub node_name: String,
    /// The output type.
    pub output_type: Type,
    /// The body of the step function.
    pub body: Vec<Statement>,
    /// The update of the node's state.
    pub state_elements_step: Vec<StateElementStep>,
    /// The output expression.
    pub output_expression: Expression,
}

/// A state element structure for the step update.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct StateElementStep {
    /// The name of the memory storage.
    pub identifier: String,
    /// The expression that will update the memory.
    pub expression: Expression,
}
