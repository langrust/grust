//! LIR [StateMachine] module.

prelude! {
    ast::contract::Term,
}

/// A state element structure.
///
/// The type parameter is the type of the data for the `Self::Buffer` case.
#[derive(Debug, PartialEq)]
pub enum StateElm<T> {
    /// A buffer identifier and some data.
    Buffer {
        /// Identifier of the buffer.
        ident: String,
        /// Buffer data.
        data: T,
    },
    /// A node.
    CalledNode {
        /// Identifier of the memory storage.
        memory_ident: String,
        /// Name of the node called.
        node_name: String,
    },
}

mk_new! { impl{T} StateElm<T> =>
    Buffer : buffer {
        ident : impl Into<String> = ident.into(),
        data : T,
    }
    CalledNode : called_node {
        memory_ident : impl Into<String> = memory_ident.into(),
        node_name : impl Into<String> = node_name.into(),
    }
}

pub type StateElmInfo = StateElm<Typ>;

mk_new! { impl StateElmInfo =>
    Buffer : buffer_info {
        ident : impl Into<String> = ident.into(),
        data : Typ,
    }
    CalledNode : called_node_info {
        memory_ident : impl Into<String> = memory_ident.into(),
        node_name : impl Into<String> = node_name.into(),
    }
}

pub type StateElmInit = StateElm<Expr>;

mk_new! { impl StateElmInit =>
    Buffer : buffer_init {
        ident : impl Into<String> = ident.into(),
        data : Expr,
    }
    CalledNode : called_node_init {
        memory_ident : impl Into<String> = memory_ident.into(),
        node_name : impl Into<String> = node_name.into(),
    }
}

/// An input element structure.
#[derive(Debug, PartialEq)]
pub struct InputElm {
    /// The name of the input.
    pub identifier: String,
    /// The type of the input.
    pub typ: Typ,
}

mk_new! { impl InputElm =>
    new {
        identifier : impl Into<String> = identifier.into(),
        typ : Typ,
    }
}

/// A node input structure.
#[derive(Debug, PartialEq)]
pub struct Input {
    /// The node's name.
    pub node_name: String,
    /// The input's elements.
    pub elements: Vec<InputElm>,
}

mk_new! { impl Input =>
    new {
        node_name : impl Into<String> = node_name.into(),
        elements : Vec<InputElm>,
    }
}

/// A init function.
#[derive(Debug, PartialEq)]
pub struct Init {
    /// The node's name.
    pub node_name: String,
    /// The initialization of the node's state.
    pub state_init: Vec<StateElmInit>,
    /// The invariant initialization to prove.
    pub invariant_initialization: Vec<Term>,
}

mk_new! { impl Init =>
    new {
        node_name : impl Into<String> = node_name.into(),
        state_init : Vec<StateElmInit>,
        invariant_initialization : Vec<Term>,
    }
}

/// A step function.
#[derive(Debug, PartialEq)]
pub struct Step {
    /// The node's name.
    pub node_name: String,
    /// The output type.
    pub output_type: Typ,
    /// The body of the step function.
    pub body: Vec<Stmt>,
    /// The update of the node's state.
    pub state_elements_step: Vec<StateElmStep>,
    /// The output expression.
    pub output_expression: Expr,
    /// The contract to prove.
    pub contract: Contract,
}

mk_new! { impl Step =>
    new {
        node_name: impl Into<String> = node_name.into(),
        output_type: Typ,
        body: Vec<Stmt>,
        state_elements_step: Vec<StateElmStep>,
        output_expression: Expr,
        contract: Contract,
    }
}

/// A state element structure for the step update.
#[derive(Debug, PartialEq)]
pub struct StateElmStep {
    /// The name of the memory storage.
    pub identifier: String,
    /// The expression that will update the memory.
    pub expression: Expr,
}

mk_new! { impl StateElmStep =>
    new {
        identifier: impl Into<String> = identifier.into(),
        expression: Expr,
    }
}

/// A node state structure.
#[derive(Debug, PartialEq)]
pub struct State {
    /// The node's name.
    pub node_name: String,
    /// The state's elements.
    pub elements: Vec<StateElmInfo>,
    /// The init function.
    pub init: Init,
    /// The step function.
    pub step: Step,
}

mk_new! { impl State => new {
    node_name : impl Into<String> = node_name.into(),
    elements : Vec<StateElmInfo>,
    init : Init,
    step : Step,
} }

/// A state-machine structure.
#[derive(Debug, PartialEq)]
pub struct StateMachine {
    /// The node's name.
    pub name: String,
    /// The input structure.
    pub input: Input,
    /// The state structure.
    pub state: State,
}

mk_new! { impl StateMachine => new {
    name : impl Into<String> = name.into(),
    input : Input,
    state : State,
} }
