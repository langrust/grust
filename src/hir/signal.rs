use crate::common::scope::Scope;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust signal HIR.
pub struct Signal {
    /// Signal identifier.
    pub id: String,
    /// Signal scope.
    pub scope: Scope,
}
