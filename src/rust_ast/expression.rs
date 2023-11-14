use crate::common::{
    constant::Constant,
    operator::{BinaryOperator, UnaryOperator},
};

use super::{block::Block, pattern::Pattern, r#type::Type};

/// Rust expressions.
#[derive(Debug, PartialEq, serde::Serialize)]
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
    /// An array creation: `[1, 2, x]`.
    Array {
        /// The elements.
        elements: Vec<Expression>,
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
    /// A macro call: `vec![1, 2, 3]`.
    Macro {
        /// The macro called.
        r#macro: String,
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
    /// An async block expression: `async { my_future.await }`.
    Async {
        /// Move used element: `true` is move, `false` is normal.
        r#move: bool,
        /// The body of the async block.
        body: Block,
    },
    /// An awit expression: `my_future.await`.
    Await {
        /// The expression awited.
        expression: Box<Expression>,
    },
    /// A tuple expression: `(x, y)`
    Tuple {
        /// Elements of the tuple.
        elements: Vec<Expression>,
    },
    /// An if_then_else expression: `if test { "ok" } else { "oh no" }`.
    IfThenElse {
        /// The test expression.
        condition: Box<Expression>,
        /// The `true` block.
        then_branch: Block,
        /// The `false` block.
        else_branch: Option<Block>,
    },
    /// A match expression: `match c { Color::Blue => 1, _ => 0, }`
    Match {
        /// The matched expression.
        matched: Box<Expression>,
        /// The pattern matching arms.
        arms: Vec<Arm>,
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
                write!(f, "{} {{ {} }}", name, fields)
            }
            Expression::Array { elements } => {
                let elements = elements
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "[{}]", elements)
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
            Expression::Macro { r#macro, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}!({})", r#macro, arguments)
            }
            Expression::Unary {
                operator,
                expression,
            } => match operator {
                UnaryOperator::Neg => write!(f, "-{expression}"),
                UnaryOperator::Not => write!(f, "!{expression}"),
                UnaryOperator::Brackets => write!(f, "({expression})"),
            },
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
                let inputs = inputs
                    .iter()
                    .map(|input| format!("{input}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(output) = output {
                    if let Expression::Block { .. } = body.as_ref() {
                        write!(f, "{}|{}| -> {} {}", r#move, inputs, output, body)
                    } else {
                        write!(f, "{}|{}| -> {} {{ {} }}", r#move, inputs, output, body)
                    }
                } else {
                    write!(f, "{}|{}| {}", r#move, inputs, body)
                }
            }
            Expression::Async { r#move, body } => {
                let r#move = if *r#move { "move " } else { "" };
                write!(f, "async {}{}", r#move, body)
            }
            Expression::Await { expression } => write!(f, "{}.await", expression),
            Expression::Tuple { elements } => {
                let elements = elements
                    .iter()
                    .map(|element| format!("{element}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", elements)
            }
            Expression::IfThenElse {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_branch = if let Some(else_branch) = else_branch {
                    format!(" else {else_branch}")
                } else {
                    "".to_string()
                };
                write!(f, "if {} {}{}", condition, then_branch, else_branch)
            }
            Expression::Match { matched, arms } => {
                let arms = arms
                    .iter()
                    .map(|arm| format!("{arm}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "match {} {{ {} }}", matched, arms)
            }
        }
    }
}

/// A structure's field filled with an expression.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct FieldExpression {
    /// Name of the field.
    pub name: String,
    /// Expression stored in the field.
    pub expression: Expression,
}

impl std::fmt::Display for FieldExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.expression.to_string() == self.name {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.expression)
        }
    }
}

/// An arm in a match expression: `Point { x: 0, y } if y > 0 => y,`
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Arm {
    /// The pattern matching.
    pub pattern: Pattern,
    /// An optional guard.
    pub guard: Option<Expression>,
    /// The body of the arm.
    pub body: Expression,
}

impl std::fmt::Display for Arm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = if let Some(guard) = &self.guard {
            format!(" if {guard}")
        } else {
            "".to_string()
        };
        write!(f, "{}{} => {},", self.pattern, guard, self.body)
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{
            constant::Constant,
            operator::{BinaryOperator, UnaryOperator},
        },
        rust_ast::{
            block::Block,
            expression::{Arm, FieldExpression},
            pattern::Pattern,
            r#type::Type,
            statement::{r#let::Let, Statement},
        },
    };

    use super::Expression;

    #[test]
    fn should_format_literal_expression() {
        let expression = Expression::Literal {
            literal: Constant::Integer(1),
        };
        let control = String::from("1i64");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_identifier_expression() {
        let expression = Expression::Identifier {
            identifier: String::from("x"),
        };
        let control = String::from("x");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_tuple_expression() {
        let expression = Expression::Tuple {
            elements: vec![
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
                Expression::Identifier {
                    identifier: String::from("y"),
                },
            ],
        };
        let control = String::from("(1i64, y)");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_array_expression() {
        let expression = Expression::Array {
            elements: vec![
                Expression::Literal {
                    literal: Constant::Integer(1),
                },
                Expression::Identifier {
                    identifier: String::from("y"),
                },
            ],
        };
        let control = String::from("[1i64, y]");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_structure_expression() {
        let expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                FieldExpression {
                    name: String::from("x"),
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                FieldExpression {
                    name: String::from("y"),
                    expression: Expression::Identifier {
                        identifier: String::from("y"),
                    },
                },
            ],
        };
        let control = String::from("Point { x: 1i64, y }");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_unary_expression() {
        let expression = Expression::Unary {
            operator: UnaryOperator::Neg,
            expression: Box::new(Expression::Identifier {
                identifier: String::from("x"),
            }),
        };
        let control = String::from("-x");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_binary_expression() {
        let expression = Expression::Binary {
            left: Box::new(Expression::Identifier {
                identifier: String::from("x"),
            }),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal {
                literal: Constant::Integer(1),
            }),
        };
        let control = String::from("x + 1i64");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_assignement_expression() {
        let expression = Expression::Assignement {
            left: Box::new(Expression::Identifier {
                identifier: String::from("x"),
            }),
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Identifier {
                    identifier: String::from("x"),
                }),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Literal {
                    literal: Constant::Integer(1),
                }),
            }),
        };
        let control = String::from("x = x + 1i64");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_block_expression() {
        let expression = Expression::Block {
            block: Block {
                statements: vec![
                    Statement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: true,
                            identifier: String::from("x"),
                        },
                        expression: Expression::Literal {
                            literal: Constant::Integer(1),
                        },
                    }),
                    Statement::ExpressionIntern(Expression::Assignement {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("x"),
                        }),
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("x"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Literal {
                                literal: Constant::Integer(1),
                            }),
                        }),
                    }),
                    Statement::ExpressionLast(Expression::Identifier {
                        identifier: String::from("x"),
                    }),
                ],
            },
        };
        let control = String::from("{ let mut x = 1i64; x = x + 1i64; x }");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_if_then_else_expression() {
        let expression = Expression::IfThenElse {
            condition: Box::new(Expression::Identifier {
                identifier: String::from("test"),
            }),
            then_branch: Block {
                statements: vec![
                    Statement::ExpressionIntern(Expression::Assignement {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("x"),
                        }),
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("x"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Literal {
                                literal: Constant::Integer(1),
                            }),
                        }),
                    }),
                    Statement::ExpressionLast(Expression::Identifier {
                        identifier: String::from("x"),
                    }),
                ],
            },
            else_branch: Some(Block {
                statements: vec![
                    Statement::ExpressionIntern(Expression::Assignement {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("x"),
                        }),
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("x"),
                            }),
                            operator: BinaryOperator::Mul,
                            right: Box::new(Expression::Literal {
                                literal: Constant::Integer(2),
                            }),
                        }),
                    }),
                    Statement::ExpressionLast(Expression::Identifier {
                        identifier: String::from("x"),
                    }),
                ],
            }),
        };
        let control = String::from("if test { x = x + 1i64; x } else { x = x * 2i64; x }");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_function_call_expression() {
        let expression = Expression::FunctionCall {
            function: Box::new(Expression::Identifier {
                identifier: String::from("foo"),
            }),
            arguments: vec![
                Expression::Identifier {
                    identifier: String::from("a"),
                },
                Expression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };
        let control = String::from("foo(a, b)");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_method_call_expression() {
        let expression = Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                identifier: String::from("a"),
            }),
            method: String::from("foo"),
            arguments: vec![Expression::Identifier {
                identifier: String::from("b"),
            }],
        };
        let control = String::from("a.foo(b)");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_macro_call_expression() {
        let expression = Expression::Macro {
            r#macro: String::from("vec"),
            arguments: vec![
                Expression::Identifier {
                    identifier: String::from("a"),
                },
                Expression::Identifier {
                    identifier: String::from("b"),
                },
            ],
        };
        let control = String::from("vec!(a, b)");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_field_access_expression() {
        let expression = Expression::FieldAccess {
            expression: Box::new(Expression::Identifier {
                identifier: String::from("my_point"),
            }),
            field: String::from("x"),
        };
        let control = String::from("my_point.x");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_reference_expression() {
        let expression = Expression::Reference {
            mutable: true,
            expression: Box::new(Expression::Identifier {
                identifier: String::from("x"),
            }),
        };
        let control = String::from("&mut x");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_closure_expression() {
        let expression = Expression::Closure {
            r#move: true,
            inputs: vec![Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier: String::from("a"),
            }],
            output: Some(Type::Identifier {
                identifier: String::from("i64"),
            }),
            body: Box::new(Expression::Binary {
                left: Box::new(Expression::Identifier {
                    identifier: String::from("x"),
                }),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Identifier {
                    identifier: String::from("a"),
                }),
            }),
        };
        let control = String::from("move |a| -> i64 { x + a }");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_await_expression() {
        let expression = Expression::Await {
            expression: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    identifier: String::from("get_message"),
                }),
                arguments: vec![],
            }),
        };
        let control = String::from("get_message().await");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_async_expression() {
        let expression = Expression::Async {
            r#move: false,
            body: Block {
                statements: vec![
                    Statement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: true,
                            identifier: String::from("x"),
                        },
                        expression: Expression::Await {
                            expression: Box::new(Expression::FunctionCall {
                                function: Box::new(Expression::Identifier {
                                    identifier: String::from("get_message"),
                                }),
                                arguments: vec![],
                            }),
                        },
                    }),
                    Statement::ExpressionIntern(Expression::Assignement {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("x"),
                        }),
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("x"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Literal {
                                literal: Constant::Integer(1),
                            }),
                        }),
                    }),
                    Statement::ExpressionLast(Expression::Identifier {
                        identifier: String::from("x"),
                    }),
                ],
            },
        };
        let control = String::from("async { let mut x = get_message().await; x = x + 1i64; x }");
        assert_eq!(format!("{}", expression), control)
    }

    #[test]
    fn should_format_match_expression() {
        let expression = Expression::Match {
            matched: Box::new(Expression::Identifier {
                identifier: String::from("c"),
            }),
            arms: vec![
                Arm {
                    pattern: Pattern::Literal {
                        literal: Constant::Enumeration(String::from("Color"), String::from("Blue")),
                    },
                    guard: None,
                    body: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                Arm {
                    pattern: Pattern::Default,
                    guard: None,
                    body: Expression::Literal {
                        literal: Constant::Integer(0),
                    },
                },
            ],
        };
        let control = String::from("match c { Color::Blue => 1i64, _ => 0i64, }");
        assert_eq!(format!("{}", expression), control)
    }
}
