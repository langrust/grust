use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{
    dependencies::Dependencies, equation::Equation, expression::Expression, pattern::Pattern,
};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression HIR.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Signal call stream expression.
    SignalCall {
        /// Signal's id in Symbol Table.
        id: usize,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
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
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Map application stream expression.
    FunctionApplication {
        /// The expression applied.
        function_expression: Expression,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Node application stream expression.
    NodeApplication {
        /// Node's id in Symbol Table.
        node_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, StreamExpression)>,
        /// Output signal's id in Symbol Table.
        output_id: usize,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Unitary node application stream expression.
    UnitaryNodeApplication {
        /// The unitary node id in Symbol Table.
        unitary_node_id: usize,
        /// The original node's id in Symbol Table.
        node_id: usize,
        /// Output signal's id in Symbol Table.
        output_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, StreamExpression)>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
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
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Array stream expression.
    Array {
        /// The elements inside the array.
        elements: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Pattern matching stream expression.
    Match {
        /// The stream expression to match.
        expression: Box<StreamExpression>,
        /// The different matching cases.
        arms: Vec<(
            Pattern,
            Option<StreamExpression>,
            Vec<Equation>,
            StreamExpression,
        )>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// When present stream expression.
    When {
        /// The identifier of the value when present
        id: usize,
        /// The optional stream expression.
        option: Box<StreamExpression>,
        /// The body of present case when normalized.
        present_body: Vec<Equation>,
        /// The stream expression when present.
        present: Box<StreamExpression>,
        /// The body of default case when normalized.
        default_body: Vec<Equation>,
        /// The default stream expression.
        default: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
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
        /// Stream expression dependencies.
        dependencies: Dependencies,
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
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Array map operator expression.
    Map {
        /// The array expression.
        expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
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
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Array sort operator expression.
    Sort {
        /// The array expression.
        expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Array zip operator expression.
    Zip {
        /// The array expressions.
        arrays: Vec<StreamExpression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
}
