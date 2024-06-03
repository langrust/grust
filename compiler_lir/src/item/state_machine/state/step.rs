prelude! {}

/// A step function.
#[derive(Debug, PartialEq)]
pub struct Step {
    /// The node's name.
    pub node_name: String,
    /// The input's generic types.
    pub generics: Vec<(String, Typ)>,
    /// The output type.
    pub output_type: Typ,
    /// The body of the step function.
    pub body: Vec<Stmt>,
    /// The update of the node's state.
    pub state_elements_step: Vec<StateElementStep>,
    /// The output expression.
    pub output_expression: Expr,
    /// The contract to prove.
    pub contract: Contract,
}

mk_new! { impl Step =>
    new {
        node_name: impl Into<String> = node_name.into(),
        generics: Vec<(String, Typ)>,
        output_type: Typ,
        body: Vec<Stmt>,
        state_elements_step: Vec<StateElementStep>,
        output_expression: Expr,
        contract: Contract,
    }
}

/// A state element structure for the step update.
#[derive(Debug, PartialEq)]
pub struct StateElementStep {
    /// The name of the memory storage.
    pub identifier: String,
    /// The expression that will update the memory.
    pub expression: Expr,
}

mk_new! { impl StateElementStep =>
    new {
        identifier: impl Into<String> = identifier.into(),
        expression: Expr,
    }
}
