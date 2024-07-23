//! LIR [Expr] module.

prelude! {
    operator::{BinaryOperator, UnaryOperator},
    Pattern, Block,
}

/// LIR expressions.
#[derive(Debug, PartialEq)]
pub enum Expr {
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
    /// An unitary operation: `!x`.
    Unop {
        /// The operator.
        op: UnaryOperator,
        /// The expression.
        expression: Box<Self>,
    },
    /// A binary operation: `x + y`.
    Binop {
        /// The operator.
        op: BinaryOperator,
        /// The left expression.
        left_expression: Box<Self>,
        /// The right expression.
        right_expression: Box<Self>,
    },
    /// An if_then_else expression: `if test { "ok" } else { "oh no" }`.
    IfThenElse {
        /// The test expression.
        condition: Box<Self>,
        /// The `true` block.
        then_branch: Block,
        /// The `false` block.
        else_branch: Block,
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
        fields: Vec<(String, Self)>,
    },
    /// A enumeration literal expression: `Color::Red`.
    Enumeration {
        /// The name of the enumeration.
        name: String,
        /// The name of the element.
        element: String,
    },
    /// An array expression: `[1, 2, 3]`.
    Array {
        /// The elements inside the array.
        elements: Vec<Self>,
    },
    /// A tuple expression: `(1, 2, 3)`.
    Tuple {
        /// The elements inside the tuple.
        elements: Vec<Self>,
    },
    /// A block scope: `{ let x = 1; x }`.
    Block {
        /// The block.
        block: Block,
    },
    /// A function call: `foo(x, y)`.
    FunctionCall {
        /// The function called.
        function: Box<Self>,
        /// The arguments.
        arguments: Vec<Self>,
    },
    /// A node call: `self.called_node.step(inputs)`.
    NodeCall {
        /// The identifier to the node.
        node_identifier: String,
        /// The name of the input structure of the called node.
        input_name: String,
        /// The filled input's fields.
        input_fields: Vec<(String, Self)>,
    },
    /// A named or unamed field access: `my_point.x`.
    FieldAccess {
        /// The structure or tuple typed expression.
        expression: Box<Self>,
        /// The identifier of the field.
        field: FieldIdentifier,
    },
    /// A lambda expression: `|x, y| x * y`.
    Lambda {
        /// The lambda inputs.
        inputs: Vec<(String, Typ)>,
        /// The output type.
        output: Typ,
        /// The body of the closure.
        body: Box<Self>,
    },
    /// A match expression: `match c { Color::Blue => 1, _ => 0, }`
    Match {
        /// The matched expression.
        matched: Box<Self>,
        /// The pattern matching arms.
        arms: Vec<(Pattern, Option<Self>, Self)>,
    },
    /// A map expression: `my_list.map(|x| x + 1)`
    Map {
        /// The mapped expression.
        mapped: Box<Self>,
        /// The mapping function.
        function: Box<Self>,
    },
    /// A fold expression: `my_list.fold(0, |sum, x| x + sum)`
    Fold {
        /// The folded expression.
        folded: Box<Self>,
        /// The initialization expression.
        initialization: Box<Self>,
        /// The folding function.
        function: Box<Self>,
    },
    /// A sort expression: `my_list.map(|a, b| a - b)`
    Sort {
        /// The sorted expression.
        sorted: Box<Self>,
        /// The sorting function.
        function: Box<Self>,
    },
    /// Arrays zip operator expression: `zip(a, b, [1, 2, 3])`
    Zip {
        /// The arrays expression.
        arrays: Vec<Self>,
    },
    /// Into method.
    IntoMethod {
        /// The expression.
        expression: Box<Self>,
    },
}

impl Expr {
    mk_new! {
        Literal: literal { literal: Constant }
        Literal: lit { literal: Constant }
        Identifier: ident { identifier: impl Into<String> = identifier.into() }
        Unop: unop {
            op: UnaryOperator,
            expression: Self = Box::new(expression),
        }
        Binop: binop {
            op: BinaryOperator,
            left_expression: Self = left_expression.into(),
            right_expression: Self = right_expression.into(),
        }
        IfThenElse: ite {
            condition: Self = Box::new(condition),
            then_branch: Block,
            else_branch: Block,
        }
        MemoryAccess: memory_access { identifier: impl Into<String> = identifier.into() }
        InputAccess: input_access { identifier: impl Into<String> = identifier.into() }
        Structure: structure {
            name: impl Into<String> = name.into(),
            fields: Vec<(String, Self)>
        }
        Enumeration: enumeration {
            name: impl Into<String> = name.into(),
            element: impl Into<String> = element.into(),
        }
        Array: array { elements: Vec<Self> }
        Tuple: tuple { elements: Vec<Self> }
        Block: block { block: Block }
        FunctionCall: function_call {
            function: Self = function.into(),
            arguments: Vec<Self>,
        }
        NodeCall: node_call {
            node_identifier: impl Into<String> = node_identifier.into(),
            input_name: impl Into<String> = input_name.into(),
            input_fields: Vec<(String, Self)>,
        }
        FieldAccess: field_access {
            expression: Self = expression.into(),
            field: FieldIdentifier
        }
        Lambda: lambda {
            inputs: Vec<(String, Typ)>,
            output: Typ,
            body: Self = body.into(),
        }
        Match: pat_match {
            matched: Self = matched.into(),
            arms: Vec<(Pattern, Option<Self>, Self)>
        }
        Map: map {
            mapped: Self = mapped.into(),
            function: Self = function.into(),
        }
        Fold: fold {
            folded: Self = folded.into(),
            initialization: Self = initialization.into(),
            function: Self = function.into(),
        }
        Sort: sort {
            sorted: Self = sorted.into(),
            function: Self = function.into()
        }
        Zip: zip { arrays: Vec<Self> }
        IntoMethod: into_call { expression: Self = expression.into() }
    }

    /// True on expressions that require parens to be used as a function in a function call.
    ///
    /// More precisely assume a call like `<expr>(<params>)`, then this function returns `true` iff
    /// `<expr>` should be wrapped in parens for the whole call to be legal rust.
    pub fn as_function_requires_parens(&self) -> bool {
        use Expr::*;
        match self {
            Literal { .. }
            | Identifier { .. }
            | MemoryAccess { .. }
            | InputAccess { .. }
            | Enumeration { .. }
            | Array { .. }
            | Tuple { .. }
            | Block { .. }
            | FieldAccess { .. } => false,
            Unop { .. }
            | Binop { .. }
            | IfThenElse { .. }
            | Structure { .. }
            | FunctionCall { .. }
            | NodeCall { .. }
            | Lambda { .. }
            | Match { .. }
            | Map { .. }
            | Fold { .. }
            | Sort { .. }
            | Zip { .. }
            | IntoMethod { .. } => true,
        }
    }
}

/// LIR field access member.
#[derive(Debug, PartialEq)]
pub enum FieldIdentifier {
    /// Named field access.
    Named(String),
    /// Unamed field access.
    Unamed(usize),
}

impl FieldIdentifier {
    pub fn named(s: impl Into<String>) -> Self {
        Self::Named(s.into())
    }
    pub fn unamed(n: usize) -> Self {
        Self::Unamed(n)
    }
}
