use crate::ast::{expression::Expression, pattern::Pattern};
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{dependencies::Dependencies, equation::Equation, signal::Signal};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression HIR.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Signal call stream expression.
    SignalCall {
        /// The called signal.
        signal: Signal,
        /// Stream Expression type.
        typing: Type,
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
        typing: Type,
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
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
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
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// Unitary node application stream expression.
    UnitaryNodeApplication {
        /// The node state identifier.
        id: Option<String>,
        /// The mother node type.
        node: String,
        /// The output signal corresponding to the unitary node.
        signal: String,
        /// The inputs to the expression.
        inputs: Vec<(String, StreamExpression)>,
        /// Stream Expression type.
        typing: Type,
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
        typing: Type,
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
        typing: Type,
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
        typing: Type,
        /// Stream expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
    /// When present stream expression.
    When {
        /// The identifier of the value when present
        id: String,
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
        typing: Type,
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
        typing: Type,
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
        typing: Type,
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
        typing: Type,
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
        typing: Type,
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
        typing: Type,
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
        typing: Type,
        /// Expression location.
        location: Location,
        /// Stream expression dependencies.
        dependencies: Dependencies,
    },
}

impl StreamExpression {
    /// Get the reference to the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::new(),
    /// };
    /// let typing = stream_expression.get_type();
    /// assert_eq!(typing, &Type::Integer)
    /// ```
    pub fn get_type(&self) -> &Type {
        match self {
            StreamExpression::Constant { typing, .. }
            | StreamExpression::SignalCall { typing, .. }
            | StreamExpression::FollowedBy { typing, .. }
            | StreamExpression::FunctionApplication { typing, .. }
            | StreamExpression::NodeApplication { typing, .. }
            | StreamExpression::UnitaryNodeApplication { typing, .. }
            | StreamExpression::Structure { typing, .. }
            | StreamExpression::Array { typing, .. }
            | StreamExpression::Match { typing, .. }
            | StreamExpression::When { typing, .. }
            | StreamExpression::FieldAccess { typing, .. }
            | StreamExpression::TupleElementAccess { typing, .. }
            | StreamExpression::Map { typing, .. }
            | StreamExpression::Fold { typing, .. }
            | StreamExpression::Sort { typing, .. }
            | StreamExpression::Zip { typing, .. } => typing,
        }
    }

    /// Get the reference to the stream expression's location.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::new(),
    /// };
    /// let location = stream_expression.get_location();
    /// assert_eq!(location, &Location::default())
    /// ```
    pub fn get_location(&self) -> &Location {
        match self {
            StreamExpression::Constant { location, .. }
            | StreamExpression::SignalCall { location, .. }
            | StreamExpression::FollowedBy { location, .. }
            | StreamExpression::FunctionApplication { location, .. }
            | StreamExpression::NodeApplication { location, .. }
            | StreamExpression::UnitaryNodeApplication { location, .. }
            | StreamExpression::Structure { location, .. }
            | StreamExpression::Array { location, .. }
            | StreamExpression::Match { location, .. }
            | StreamExpression::When { location, .. }
            | StreamExpression::FieldAccess { location, .. }
            | StreamExpression::TupleElementAccess { location, .. }
            | StreamExpression::Map { location, .. }
            | StreamExpression::Fold { location, .. }
            | StreamExpression::Sort { location, .. }
            | StreamExpression::Zip { location, .. } => location,
        }
    }

    /// Get the reference to the stream expression's dependencies.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    ///     dependencies: Dependencies::from(vec![]),
    /// };
    /// let dependencies = stream_expression.get_dependencies();
    /// assert_eq!(*dependencies, vec![])
    /// ```
    pub fn get_dependencies(&self) -> &Vec<(String, usize)> {
        match self {
            StreamExpression::Constant { dependencies, .. }
            | StreamExpression::SignalCall { dependencies, .. }
            | StreamExpression::FollowedBy { dependencies, .. }
            | StreamExpression::FunctionApplication { dependencies, .. }
            | StreamExpression::NodeApplication { dependencies, .. }
            | StreamExpression::UnitaryNodeApplication { dependencies, .. }
            | StreamExpression::Structure { dependencies, .. }
            | StreamExpression::Array { dependencies, .. }
            | StreamExpression::Match { dependencies, .. }
            | StreamExpression::When { dependencies, .. }
            | StreamExpression::FieldAccess { dependencies, .. }
            | StreamExpression::TupleElementAccess { dependencies, .. }
            | StreamExpression::Map { dependencies, .. }
            | StreamExpression::Fold { dependencies, .. }
            | StreamExpression::Sort { dependencies, .. }
            | StreamExpression::Zip { dependencies, .. } => dependencies.get().unwrap(),
        }
    }
}
