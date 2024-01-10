use crate::{
    ast::pattern::Pattern,
    common::{constant::Constant, r#type::Type},
};

use super::block::Block;

/// LIR expressions.
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
    /// A memory access: `self.i_mem`.
    MemoryAccess {
        /// The identifier to the memory.
        identifier: String,
    },
    /// An input access: `self.i_mem`.
    InputAccess {
        /// The identifier to the input.
        identifier: String,
    },
    /// A structure literal expression: `Point { x: 1, y: 1 }`.
    Structure {
        /// The name of the structure.
        name: String,
        /// The filled fields.
        fields: Vec<(String, Expression)>,
    },
    /// An array expression: `[1, 2, 3]`.
    Array {
        /// The elements inside the array.
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
    /// A node call: `self.called_node.step(inputs)`.
    NodeCall {
        /// The identifier to the node.
        node_identifier: String,
        /// The name of the input structure of the called node.
        input_name: String,
        /// The filled input's fields.
        input_fields: Vec<(String, Expression)>,
    },
    /// A named or unamed field access: `my_point.x`.
    FieldAccess {
        /// The structure or tuple typed expression.
        expression: Box<Expression>,
        /// The identifier of the field.
        field: FieldIdentifier,
    },
    /// A lambda expression: `|x, y| x * y`.
    Lambda {
        /// The lambda inputs.
        inputs: Vec<(String, Type)>,
        /// The output type.
        output: Type,
        /// The body of the closure.
        body: Box<Expression>,
    },
    /// An if_then_else expression: `if test { "ok" } else { "oh no" }`.
    IfThenElse {
        /// The test expression.
        condition: Box<Expression>,
        /// The `true` block.
        then_branch: Block,
        /// The `false` block.
        else_branch: Block,
    },
    /// A match expression: `match c { Color::Blue => 1, _ => 0, }`
    Match {
        /// The matched expression.
        matched: Box<Expression>,
        /// The pattern matching arms.
        arms: Vec<(Pattern, Option<Expression>, Expression)>,
    },
    /// A map expression: `my_list.map(|x| x + 1)`
    Map {
        /// The mapped expression.
        mapped: Box<Expression>,
        /// The mapping function.
        function: Box<Expression>,
    },
    /// A fold expression: `my_list.fold(0, |sum, x| x + sum)`
    Fold {
        /// The folded expression.
        folded: Box<Expression>,
        /// The initialization expression.
        initialization: Box<Expression>,
        /// The folding function.
        function: Box<Expression>,
    },
    /// A sort expression: `my_list.map(|a, b| a - b)`
    Sort {
        /// The sorted expression.
        sorted: Box<Expression>,
        /// The sorting function.
        function: Box<Expression>,
    },
    /// Arrays zip operator expression: `zip(a, b, [1, 2, 3])`
    Zip {
        /// The arrays expression.
        arrays: Vec<Expression>,
    },
}

/// LIR field access member.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum FieldIdentifier {
    /// Named field access.
    Named(String),
    /// Unamed field access.
    Unamed(usize),
}
