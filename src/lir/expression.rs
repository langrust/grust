use crate::common::{
    constant::Constant,
    operator::{BinaryOperator, UnaryOperator},
};

use super::{block::Block, pattern::Pattern, r#type::Type};

/// Rust expressions.
pub enum Expression {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: String,
    },
    /// A structure literal expression: `Point { x: 1, y: 1 }`.
    Structure {
        /// The name of the structure.
        name: String,
        /// The filled fields.
        fields: Vec<FieldExpression>,
    },
    /// A block scope: `{ let x = 1; x }`.
    Block {
        /// The block.
        block: Block,
    },
    /// A function call: `foo(x, y)`.
    FunctionCall {
        /// The function called.
        function: Box<Expression>,
        /// The arguments.
        arguments: Vec<Expression>,
    },
    /// A method call: `a.foo(x, y)`.
    MethodCall {
        /// The receiver which perform the method.
        receiver: Box<Expression>,
        /// The method called.
        method: String,
        /// The arguments.
        arguments: Vec<Expression>,
    },
    /// An unary operation: `-x`.
    Unary {
        /// The operator.
        operator: UnaryOperator,
        /// The expression.
        expression: Box<Expression>,
    },
    /// A binary operation: `x + y`.
    Binary {
        /// The left expression.
        left: Box<Expression>,
        /// The operator.
        operator: BinaryOperator,
        /// The right expression.
        right: Box<Expression>,
    },
    /// An assignement expression: `x = y + 1`.
    Assignement {
        /// The receiver.
        left: Box<Expression>,
        /// The expression assigned to the receiver.
        right: Box<Expression>,
    },
    /// A field access: `my_point.x`.
    FieldAccess {
        /// The structure typed expression.
        expression: Box<Expression>,
        /// The identifier of the field.
        field: String,
    },
    /// A reference: `&mut x`.
    Reference {
        /// Mutability: `true` is mutable, `false` is immutable.
        mutable: bool,
        /// The referenced expression.
        expression: Box<Expression>,
    },
    /// A closure expression: `|x, y| x * y`.
    Closure {
        /// Move used element: `true` is move, `false` is normal.
        r#move: bool,
        /// The closure inputs as a pattern.
        inputs: Vec<Pattern>,
        /// The optional output type.
        output: Option<Type>,
        /// The body of the closure.
        body: Box<Expression>,
    },
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Block { block } => write!(f, "{block}"),
            Expression::Literal { literal } => write!(f, "{literal}"),
            Expression::Identifier { identifier } => write!(f, "{identifier}"),
            Expression::Structure { name, fields } => {
                let fields = fields
                    .iter()
                    .map(|field| format!("{field}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{} {{{}}}", name, fields)
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}({})", function, arguments)
            }
            Expression::MethodCall {
                receiver,
                method,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}.{}({})", receiver, method, arguments)
            }
            Expression::Unary {
                operator,
                expression,
            } => write!(f, "{}{expression}", operator.to_string()),
            Expression::Binary {
                left,
                operator,
                right,
            } => write!(f, "{left}{}{right}", operator.to_string()),
            Expression::Assignement { left, right } => write!(f, "{left} = {right}"),
            Expression::FieldAccess { expression, field } => write!(f, "{expression}.{field}"),
            Expression::Reference {
                mutable,
                expression,
            } => {
                let mutable = if *mutable { "mut " } else { "" };
                write!(f, "&{}{}", mutable, expression)
            }
            Expression::Closure {
                r#move,
                inputs,
                output,
                body,
            } => {
                let r#move = if *r#move { "move " } else { "" };
                let inputs = if inputs.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "|{}|",
                        inputs
                            .iter()
                            .map(|input| format!("{input}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let output = if let Some(output) = output {
                    format!(" -> {output} ")
                } else {
                    "".to_string()
                };
                write!(f, "{}{}{}{}", r#move, inputs, output, body)
            }
        }
    }
}

/// A structure's field filled with an expression.
pub struct FieldExpression {
    /// Name of the field.
    pub name: String,
    /// Expression stored in the field.
    pub expression: Expression,
}

impl std::fmt::Display for FieldExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.expression)
    }
}
