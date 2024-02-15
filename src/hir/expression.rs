use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{dependencies::Dependencies, pattern::Pattern, statement::Statement};

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
        id: usize,
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
        inputs: Vec<usize>,
        /// The expression abstracted.
        expression: Box<Expression>,
    },
    /// Structure expression.
    Structure {
        /// The structure id.
        id: usize,
        /// The fields associated with their expressions.
        fields: Vec<(usize, Expression)>,
    },
    /// Enumeration expression.
    Enumeration {
        /// The enumeration id.
        enum_id: usize,
        /// The element id.
        elem_id: usize,
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
        arms: Vec<(Pattern, Option<Expression>, Vec<Statement>, Expression)>,
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
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<Expression>,
        /// The field to access.
        field: String, // can not be a usize because we don't know the structure type
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
pub struct Expression {
    /// Expression kind.
    pub kind: ExpressionKind,
    /// Expression type.
    pub typing: Option<Type>,
    /// Expression location.
    pub location: Location,
    /// Expression dependencies.
    pub dependencies: Dependencies,
}

impl Expression {
    pub fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
}
