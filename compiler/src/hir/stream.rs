//! HIR [`stream::Expr`][Expr] module.

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
        /// Node's id in memory.
        memory_id: Option<usize>,
        /// Called node's id in Symbol Table.
        called_node_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, Expr)>,
    },
    /// Detect a rising edge of the expression.
    RisingEdge {
        /// The expression to detect the rising edge from.
        expression: Box<Expr>,
    },
    /// Present event expression.
    SomeEvent {
        /// The expression of the event.
        expression: Box<Expr>,
    },
    /// Absent event expression.
    NoneEvent,
}

mk_new! { impl Kind =>
    Expression: expr { expression: expr::Kind<Expr> }
    FollowedBy: fby {
        constant: Expr = constant.into(),
        expression: Expr = expression.into(),
    }
    NodeApplication: call {
        memory_id = None,
        called_node_id: usize,
        inputs: Vec<(usize, Expr)>,
    }
    RisingEdge: rising_edge {
        expression: Expr = expression.into(),
    }
    SomeEvent: some_event {
        expression: Expr = expression.into(),
    }
    NoneEvent: none_event ()
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
            Kind::SomeEvent { expression } => expression.no_fby(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => unreachable!(),
        }
    }
    /// Tell if it is in normal form.
    ///
    /// - component application as root expression
    /// - no rising edge
    pub fn is_normal_form(&self) -> bool {
        let predicate_expression = |expression: &Expr| {
            expression.no_component_application() && expression.no_rising_edge()
        };
        let predicate_statement =
            |statement: &hir::Stmt<Expr>| statement.expression.is_normal_form();
        match &self.kind {
            Kind::Expression { expression } => {
                expression.propagate_predicate(predicate_expression, predicate_statement)
            }
            Kind::FollowedBy { expression, .. } => predicate_expression(expression),
            Kind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| predicate_expression(expression)),
            Kind::SomeEvent { expression } => predicate_expression(expression),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => false,
        }
    }
    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_component_application, |statement| {
                    statement.expression.no_component_application()
                }),
            Kind::FollowedBy { expression, .. } => expression.no_component_application(),
            Kind::NodeApplication { .. } => false,
            Kind::SomeEvent { expression } => expression.no_component_application(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { expression } => expression.no_component_application(),
        }
    }
    /// Tell if there is no rising edge.
    pub fn no_rising_edge(&self) -> bool {
        match &self.kind {
            Kind::Expression { expression } => expression
                .propagate_predicate(Self::no_rising_edge, |statement| {
                    statement.expression.no_rising_edge()
                }),
            Kind::FollowedBy { expression, .. } => expression.no_rising_edge(),
            Kind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_rising_edge()),
            Kind::SomeEvent { expression } => expression.no_rising_edge(),
            Kind::NoneEvent => true,
            Kind::RisingEdge { .. } => false,
        }
    }
}
