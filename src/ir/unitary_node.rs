use crate::common::{
    location::Location,
    type_system::Type,
};
use crate::ir::equation::Equation;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust unitary node IR.
pub struct UnitaryNode {
    /// Mother node identifier.
    pub node_id: String,
    /// Output signal identifier.
    pub output_id: String,
    /// Unitary node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Unitary node's scheduled equations.
    pub scheduled_equations: Vec<Equation>,
    /// Mother node location.
    pub location: Location,
}
