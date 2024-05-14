use crate::common::location::Location;
use crate::hir::{
    flow_expression::FlowExpression, function::Function, node::Node, statement::Statement,
    typedef::Typedef,
};

#[derive(Debug, PartialEq)]
/// A LanGRust [File] is composed of functions, nodes,
/// types defined by the user, components and interface.
pub struct File {
    /// Program types.
    pub typedefs: Vec<Typedef>,
    /// Program functions.
    pub functions: Vec<Function>,
    /// Program nodes. They are functional requirements.
    pub nodes: Vec<Node>,
    /// Program interface. It represents the system.
    pub interface: Vec<Statement<FlowExpression>>,
    /// Program location.
    pub location: Location,
}

impl File {
    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.nodes.iter().all(|node| node.no_fby())
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.nodes.iter().all(|node| node.is_normal_form())
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        self.nodes.iter().all(|node| node.no_node_application())
    }
}
