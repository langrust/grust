//! HIR [StreamExpression](crate::hir::stream_expression::StreamExpression) module.

prelude! {
    graph::Label,
    hir::{Dependencies, expr},
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression kind AST.
pub enum Kind {
    /// Expression.
    Expression {
        /// The expression kind.
        expression: expr::Kind<Expr>,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Box<Expr>,
        /// The buffered expression.
        expression: Box<Expr>,
    },
    /// Node application stream expression.
    NodeApplication {
        /// Calling node's id in Symbol Table.
        calling_node_id: usize,
        /// Called node's id in Symbol Table.
        called_node_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, Expr)>,
    },
}

mk_new! { impl Kind =>
    Expression: expr { expression: expr::Kind<Expr> }
    FollowedBy: fby {
        constant: Expr = constant.into(),
        expression: Expr = expression.into(),
    }
    NodeApplication: call {
        calling_node_id: usize,
        called_node_id: usize,
        inputs: Vec<(usize, Expr)>,
    }
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression AST.
pub struct Expr {
    /// Stream expression kind.
    pub kind: Kind,
    /// Stream expression type.
    pub typing: Option<Typ>,
    /// Stream expression location.
    pub location: Location,
    /// Stream expression dependencies.
    pub dependencies: Dependencies,
}

/// Constructs stream expression.
///
/// Typing, location and dependencies are empty.
pub fn expr(kind: Kind) -> Expr {
    Expr {
        kind,
        typing: None,
        location: Location::default(),
        dependencies: Dependencies::new(),
    }
}

impl Expr {
    /// Get stream expression's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    /// Get stream expression's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
    /// Get stream expression's dependencies.
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }

    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_fby, |statement| statement.expression.no_fby()),
            Kind::FollowedBy { .. } => false,
            Kind::NodeApplication { inputs, .. } => {
                inputs.iter().all(|(_, expression)| expression.no_fby())
            }
        }
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_node_application, |statement| {
                    statement.expression.is_normal_form()
                }),
            Kind::FollowedBy { expression, .. } => expression.no_node_application(),
            Kind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_node_application()),
        }
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_node_application, |statement| {
                    statement.expression.no_node_application()
                }),
            Kind::FollowedBy { expression, .. } => expression.no_node_application(),
            Kind::NodeApplication { .. } => false,
        }
    }
}
