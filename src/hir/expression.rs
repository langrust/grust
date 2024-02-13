
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{pattern::Pattern, statement::Statement};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Call expression.
    Call {
        /// Element identifier.
        id: usize,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Application expression.
    Application {
        /// The expression applied.
        function_expression: Box<Expression>,
        /// The inputs to the expression.
        inputs: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<usize>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Structure expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, Expression)>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array expression.
    Array {
        /// The elements inside the array.
        elements: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<Expression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<Expression>, Vec<Statement>, Expression)>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// When present expression.
    When {
        /// The identifier of the value when present
        id: usize,
        /// The optional expression.
        option: Box<Expression>,
        /// The expression when present.
        present: Box<Expression>,
        /// The body of present case when normalized.
        present_body: Vec<Statement>,
        /// The default expression.
        default: Box<Expression>,
        /// The body of present case when normalized.
        default_body: Vec<Statement>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<Expression>,
        /// The field to access.
        field: String,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Tuple element access expression.
    TupleElementAccess {
        /// The tuple expression.
        expression: Box<Expression>,
        /// The element to access.
        element_number: usize,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array map operator expression.
    Map {
        /// The array expression.
        expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array fold operator expression.
    Fold {
        /// The array expression.
        expression: Box<Expression>,
        /// The initialization expression.
        initialization_expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array sort operator expression.
    Sort {
        /// The array expression.
        expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Arrays zip operator expression.
    Zip {
        /// The array expressions.
        arrays: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
}
