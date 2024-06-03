//! HIR [Expression](crate::hir::expression::Expression) module.

prelude! {
    graph::Label,
    operator::{BinaryOperator, UnaryOperator},
    hir::{Dependencies, Pattern, Stmt},
}

#[derive(Debug, PartialEq, Clone)]
/// HIR expression kind.
pub enum Kind<E> {
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
    /// Unop expression.
    Unop {
        /// The unary operator.
        op: UnaryOperator,
        /// The input expression.
        expression: Box<E>,
    },
    /// Binop expression.
    Binop {
        /// The unary operator.
        op: BinaryOperator,
        /// The left expression.
        left_expression: Box<E>,
        /// The right expression.
        right_expression: Box<E>,
    },
    /// IfThenElse expression.
    IfThenElse {
        /// The test expression.
        expression: Box<E>,
        /// The 'true' expression.
        true_expression: Box<E>,
        /// The 'false' expression.
        false_expression: Box<E>,
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
    /// Tuple expression.
    Tuple {
        /// The elements.
        elements: Vec<E>,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<E>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<E>, Vec<Stmt<E>>, E)>,
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
        present_body: Vec<Stmt<E>>,
        /// The default expression.
        default: Box<E>,
        /// The body of present case when normalized.
        default_body: Vec<Stmt<E>>,
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

/// HIR expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    /// Expression kind.
    pub kind: Kind<Self>,
    /// Expression type.
    pub typing: Option<Typ>,
    /// Expression location.
    pub location: Location,
    /// Expression dependencies.
    pub dependencies: Dependencies,
}

impl Expr {
    /// Get expression's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    /// Get expression's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
    /// Get expression's dependencies.
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }
}

impl<E> Kind<E> {
    /// Propagate a predicate over the expression tree.
    pub fn propagate_predicate<F1, F2>(
        &self,
        predicate_expression: F1,
        predicate_statement: F2,
    ) -> bool
    where
        F1: Fn(&E) -> bool,
        F2: Fn(&Stmt<E>) -> bool,
    {
        match self {
            Kind::Constant { .. }
            | Kind::Identifier { .. }
            | Kind::Abstraction { .. }
            | Kind::Enumeration { .. } => true,
            Kind::Unop { expression, .. } => predicate_expression(expression),
            Kind::Binop {
                left_expression,
                right_expression,
                ..
            } => predicate_expression(left_expression) && predicate_expression(right_expression),
            Kind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                predicate_expression(expression)
                    && predicate_expression(true_expression)
                    && predicate_expression(false_expression)
            }
            Kind::Application {
                function_expression,
                inputs,
            } => {
                predicate_expression(function_expression)
                    && inputs
                        .iter()
                        .all(|expression| predicate_expression(expression))
            }
            Kind::Structure { fields, .. } => fields
                .iter()
                .all(|(_, expression)| predicate_expression(expression)),
            Kind::Array { elements } | Kind::Tuple { elements } => elements
                .iter()
                .all(|expression| predicate_expression(expression)),
            Kind::Match { expression, arms } => {
                predicate_expression(expression)
                    && arms.iter().all(|(_, option, body, expression)| {
                        body.iter().all(|statement| predicate_statement(statement))
                            && option
                                .as_ref()
                                .map_or(true, |expression| predicate_expression(expression))
                            && predicate_expression(expression)
                    })
            }
            Kind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                present_body
                    .iter()
                    .all(|statement| predicate_statement(statement))
                    && default_body
                        .iter()
                        .all(|statement| predicate_statement(statement))
                    && predicate_expression(option)
                    && predicate_expression(present)
                    && predicate_expression(default)
            }
            Kind::FieldAccess { expression, .. } => predicate_expression(expression),
            Kind::TupleElementAccess { expression, .. } => predicate_expression(expression),
            Kind::Map {
                expression,
                function_expression,
            } => predicate_expression(expression) && predicate_expression(function_expression),
            Kind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                predicate_expression(expression)
                    && predicate_expression(initialization_expression)
                    && predicate_expression(function_expression)
            }
            Kind::Sort {
                expression,
                function_expression,
            } => predicate_expression(expression) && predicate_expression(function_expression),
            Kind::Zip { arrays } => arrays
                .iter()
                .all(|expression| predicate_expression(expression)),
        }
    }
}
