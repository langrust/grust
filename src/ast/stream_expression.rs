use std::collections::HashMap;

use crate::ast::{expression::Expression, pattern::Pattern, typedef::Typedef};
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression AST.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Signal call stream expression.
    SignalCall {
        /// Signal identifier.
        id: String,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Constant,
        /// The buffered expression.
        expression: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Function application stream expression.
    FunctionApplication {
        /// The expression applied.
        function_expression: Expression,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Node application stream expression.
    NodeApplication {
        /// The node applied.
        node: String,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// The signal retrieved.
        signal: String,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Structure stream expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, StreamExpression)>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array stream expression.
    Array {
        /// The elements inside the array.
        elements: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Pattern matching stream expression.
    Match {
        /// The stream expression to match.
        expression: Box<StreamExpression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<StreamExpression>, StreamExpression)>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// When present stream expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional stream expression.
        option: Box<StreamExpression>,
        /// The stream expression when present.
        present: Box<StreamExpression>,
        /// The default stream expression.
        default: Box<StreamExpression>,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Field access stream expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<StreamExpression>,
        /// The field to access.
        field: String,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Tuple element access stream expression.
    TupleElementAccess {
        /// The tuple stream expression.
        expression: Box<StreamExpression>,
        /// The element to access.
        element_number: usize,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array map operator stream expression.
    Map {
        /// The array stream expression.
        expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array fold operator stream expression.
    Fold {
        /// The array stream expression.
        expression: Box<StreamExpression>,
        /// The initialization stream expression.
        initialization_expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array sort operator stream expression.
    Sort {
        /// The array stream expression.
        expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Arrays zip operator stream expression.
    Zip {
        /// The array stream expressions.
        arrays: Vec<StreamExpression>,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
}
