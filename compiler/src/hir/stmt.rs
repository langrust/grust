//! HIR [Statement](crate::hir::statement::Statement) module.

prelude! {
    hir::{Pattern, stream},
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust statement HIR.
pub struct Stmt<E> {
    /// Pattern of elements.
    pub pattern: Pattern,
    /// The expression defining the element.
    pub expression: E,
    /// Stmt location.
    pub location: Location,
}

impl Stmt<stream::Expr> {
    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.expression.no_fby()
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.expression.is_normal_form()
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        self.expression.no_node_application()
    }
}
