use crate::ast::pattern::Pattern;
use crate::common::{constant::Constant, location::Location, r#type::Type};

#[derive(Debug, PartialEq, Clone)]
/// GRust expression AST.
pub enum ExpressionKind<E> {
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
        function_expression: Box<E>,
        /// The inputs to the expression.
        inputs: Vec<E>,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<String>,
        /// The expression abstracted.
        expression: Box<E>,
    },
    /// Abstraction expression with inputs types.
    TypedAbstraction {
        /// The inputs to the abstraction.
        inputs: Vec<(String, Type)>,
        /// The expression abstracted.
        expression: Box<E>,
    },
    /// Structure expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, E)>,
    },
    /// Tuple expression.
    Tuple {
        /// The elements.
        elements: Vec<E>,
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
        elements: Vec<E>,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<E>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<E>, E)>,
    },
    /// When present expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional expression.
        option: Box<E>,
        /// The expression when present.
        present: Box<E>,
        /// The default expression.
        default: Box<E>,
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<E>,
        /// The field to access.
        field: String,
    },
    /// Tuple element access expression.
    TupleElementAccess {
        /// The tuple expression.
        expression: Box<E>,
        /// The element to access.
        element_number: usize,
    },
    /// Array map operator expression.
    Map {
        /// The array expression.
        expression: Box<E>,
        /// The function expression.
        function_expression: Box<E>,
    },
    /// Array fold operator expression.
    Fold {
        /// The array expression.
        expression: Box<E>,
        /// The initialization expression.
        initialization_expression: Box<E>,
        /// The function expression.
        function_expression: Box<E>,
    },
    /// Array sort operator expression.
    Sort {
        /// The array expression.
        expression: Box<E>,
        /// The function expression.
        function_expression: Box<E>,
    },
    /// Arrays zip operator expression.
    Zip {
        /// The array expressions.
        arrays: Vec<E>,
    },
}

#[derive(Debug, PartialEq, Clone)]
/// GRust expression AST.
pub struct Expression {
    /// Expression kind.
    pub kind: ExpressionKind<Expression>,
    /// Expression location.
    pub location: Location,
}
