use petgraph::graphmap::DiGraphMap;

use crate::common::{graph::neighbor::Label, location::Location};
use crate::hir::{
    contract::Contract, memory::Memory, statement::Statement, stream_expression::StreamExpression,
};
use crate::symbol_table::SymbolTable;

#[derive(Debug, Clone, serde::Serialize)]
/// LanGRust unitary node HIR.
pub struct UnitaryNode {
    /// The unitary node id in Symbol Table.
    pub id: usize,
    /// Unitary node's statements.
    pub statements: Vec<Statement<StreamExpression>>,
    /// Unitary node's memory.
    pub memory: Memory,
    /// Mother node location.
    pub location: Location,
    /// Unitary node dependency graph.
    pub graph: DiGraphMap<usize, Label>,
    /// Unitary node contracts.
    pub contract: Contract,
}

impl PartialEq for UnitaryNode {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements
            && self.memory == other.memory
            && self.location == other.location
            && self.eq_graph(other)
            && self.contract == other.contract
    }
}

impl UnitaryNode {
    /// Return vector of unitary node's signals id.
    pub fn get_signals_id(&self) -> Vec<usize> {
        let mut signals = vec![];
        self.statements.iter().for_each(|equation| {
            signals.push(equation.id.clone());
        });
        signals
    }

    /// Return vector of unitary node's signals name.
    pub fn get_signals_name(&self, symbol_table: &SymbolTable) -> Vec<String> {
        let mut signals = vec![];
        self.statements.iter().for_each(|equation| {
            signals.push(symbol_table.get_name(&equation.id).clone());
        });
        signals
    }

    /// Tells if two unscheduled unitary nodes are equal.
    pub fn eq_unscheduled(&self, other: &UnitaryNode) -> bool {
        self.statements.len() == other.statements.len()
            && self.statements.iter().all(|equation| {
                other
                    .statements
                    .iter()
                    .any(|other_equation| equation == other_equation)
            })
            && self.memory == other.memory
            && self.location == other.location
    }

    fn eq_graph(&self, other: &UnitaryNode) -> bool {
        let graph_nodes = self.graph.nodes();
        let other_nodes = other.graph.nodes();
        let graph_edges = self.graph.all_edges();
        let other_edges = other.graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }

    pub fn no_fby(&self) -> bool {
        self.statements.iter().all(|statement| statement.no_fby())
    }
    pub fn is_normal_form(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.is_normal_form())
    }
    pub fn no_node_application(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.no_node_application())
    }
}
