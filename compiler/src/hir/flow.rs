//! HIR [Interface](crate::hir::interface::Interface) module.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// Flow expression kinds.
pub enum Kind {
    /// Flow identifier call.
    Ident {
        /// The identifier of the flow to call.
        id: usize,
    },
    /// GReact `sample` operator.
    Sample {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Sampling period in milliseconds.
        period_ms: u64,
    },
    /// GReact `scan` operator.
    Scan {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Scaning period in milliseconds.
        period_ms: u64,
    },
    /// GReact `timeout` operator.
    Timeout {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Dealine in milliseconds.
        deadline: u64,
    },
    /// GReact `throttle` operator.
    Throttle {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Variation that will update the signal.
        delta: Constant,
    },
    /// GReact `on_change` operator.
    OnChange {
        /// Input expression.
        flow_expression: Box<Expr>,
    },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        component_id: usize,
        /// Input expressions.
        inputs: Vec<(usize, Expr)>,
    },
}

#[derive(Debug, PartialEq, Clone)]
/// Flow expression HIR.
pub struct Expr {
    /// Flow expression's kind.
    pub kind: Kind,
    /// Flow expression type.
    pub typing: Option<Typ>,
    /// Flow expression location.
    pub location: Location,
}
impl Expr {
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
}
