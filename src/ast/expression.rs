use crate::ast::pattern::Pattern;
use crate::common::{constant::Constant, location::Location, r#type::Type};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust expression AST.
pub enum ExpressionKind {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
    },
    /// Identifier expression.
    Identifier {
        /// Element identifier.
        id: String,
    },
    /// Application expression.
    Application {
        /// The expression applied.
        function_expression: Box<Expression>,
        /// The inputs to the expression.
        inputs: Vec<Expression>,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<String>,
        /// The expression abstracted.
        expression: Box<Expression>,
    },
    /// Abstraction expression with inputs types.
    TypedAbstraction {
        /// The inputs to the abstraction.
        inputs: Vec<(String, Type)>,
        /// The expression abstracted.
        expression: Box<Expression>,
    },
    /// Structure expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, Expression)>,
    },
    /// Enumeration expression.
    Enumeration {
        /// The enumeration name.
        enum_name: String,
        /// The enumeration element.
        elem_name: String,
    },
    /// Array expression.
    Array {
        /// The elements inside the array.
        elements: Vec<Expression>,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<Expression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<Expression>, Expression)>,
    },
    /// When present expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional expression.
        option: Box<Expression>,
        /// The expression when present.
        present: Box<Expression>,
        /// The default expression.
        default: Box<Expression>,
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<Expression>,
        /// The field to access.
        field: String,
    },
    /// Tuple element access expression.
    TupleElementAccess {
        /// The tuple expression.
        expression: Box<Expression>,
        /// The element to access.
        element_number: usize,
    },
    /// Array map operator expression.
    Map {
        /// The array expression.
        expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
    },
    /// Array fold operator expression.
    Fold {
        /// The array expression.
        expression: Box<Expression>,
        /// The initialization expression.
        initialization_expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
    },
    /// Array sort operator expression.
    Sort {
        /// The array expression.
        expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
    },
    /// Arrays zip operator expression.
    Zip {
        /// The array expressions.
        arrays: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust expression AST.
pub struct Expression {
    /// Expression kind.
    pub kind: ExpressionKind,
    /// Expression location.
    pub location: Location,
}
