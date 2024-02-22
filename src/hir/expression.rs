use crate::common::label::Label;
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{dependencies::Dependencies, pattern::Pattern, statement::Statement};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust expression AST.
pub enum ExpressionKind<E> {
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
        function_expression: Box<E>,
        /// The inputs to the expression.
        inputs: Vec<E>,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<usize>,
        /// The expression abstracted.
        expression: Box<E>,
    },
    /// Structure expression.
    Structure {
        /// The structure id.
        id: usize,
        /// The fields associated with their expressions.
        fields: Vec<(usize, E)>,
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
        elements: Vec<E>,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<E>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<E>, Vec<Statement<E>>, E)>,
    },
    /// When present expression.
    When {
        /// The identifier of the value when present
        id: usize,
        /// The optional expression.
        option: Box<E>,
        /// The expression when present.
        present: Box<E>,
        /// The body of present case when normalized.
        present_body: Vec<Statement<E>>,
        /// The default expression.
        default: Box<E>,
        /// The body of present case when normalized.
        default_body: Vec<Statement<E>>,
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<E>,
        /// The field to access.
        field: String, // can not be a usize because we don't know the structure type
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

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Expression {
    /// Expression kind.
    pub kind: ExpressionKind<Expression>,
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
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }
}

impl<E> ExpressionKind<E> {
    pub fn propagate_predicate<F>(&self, predicate: F) -> bool
    where
        F: Fn(&E) -> bool,
    {
        match self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Identifier { .. }
            | ExpressionKind::Abstraction { .. }
            | ExpressionKind::Enumeration { .. } => true,
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                predicate(function_expression)
                    && inputs.iter().all(|expression| predicate(expression))
            }
            ExpressionKind::Structure { fields, .. } => {
                fields.iter().all(|(_, expression)| predicate(expression))
            }
            ExpressionKind::Array { elements } => {
                elements.iter().all(|expression| predicate(expression))
            }
            ExpressionKind::Match { expression, arms } => {
                predicate(expression)
                    && arms.iter().all(|(_, option, body, expression)| {
                        debug_assert!(body.is_empty());
                        option
                            .as_ref()
                            .map_or(true, |expression| predicate(expression))
                            && predicate(expression)
                    })
            }
            ExpressionKind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                debug_assert!(present_body.is_empty());
                debug_assert!(default_body.is_empty());
                predicate(option) && predicate(present) && predicate(default)
            }
            ExpressionKind::FieldAccess { expression, .. } => predicate(expression),
            ExpressionKind::TupleElementAccess { expression, .. } => predicate(expression),
            ExpressionKind::Map {
                expression,
                function_expression,
            } => predicate(expression) && predicate(function_expression),
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                predicate(expression)
                    && predicate(initialization_expression)
                    && predicate(function_expression)
            }
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => predicate(expression) && predicate(function_expression),
            ExpressionKind::Zip { arrays } => arrays.iter().all(|expression| predicate(expression)),
        }
    }
}
