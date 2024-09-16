//! HIR [Component](crate::hir::node::Component) module.

prelude! {
    graph::*,
    hir::{Contract, Stmt, stream},
}

use super::memory::Memory;

#[derive(Debug, Clone, PartialEq)]
/// LanGRust component HIR.
pub enum Component {
    Definition(ComponentDefinition),
    Import(ComponentImport),
}
impl Component {
    pub fn get_graph(&self) -> &DiGraphMap<usize, Label> {
        match self {
            Component::Definition(comp_def) => &comp_def.graph,
            Component::Import(comp_import) => &comp_import.graph,
        }
    }
    pub fn get_reduced_graph(&self) -> &DiGraphMap<usize, Label> {
        match self {
            Component::Definition(comp_def) => &comp_def.reduced_graph,
            Component::Import(comp_import) => &comp_import.graph,
        }
    }
    pub fn get_id(&self) -> usize {
        match self {
            Component::Definition(comp_def) => comp_def.id,
            Component::Import(comp_import) => comp_import.id,
        }
    }
    pub fn get_location(&self) -> &Location {
        match self {
            Component::Definition(comp_def) => &comp_def.location,
            Component::Import(comp_import) => &comp_import.location,
        }
    }
}

#[derive(Debug, Clone)]
/// LanGRust component definition HIR.
pub struct ComponentDefinition {
    /// Component identifier.
    pub id: usize,
    /// Component's statements.
    pub statements: Vec<Stmt<stream::Expr>>,
    /// Component's contract.
    pub contract: Contract,
    /// Component location.
    pub location: Location,
    /// Component dependency graph.
    pub graph: DiGraphMap<usize, Label>,
    /// Component reduced dependency graph.
    pub reduced_graph: DiGraphMap<usize, Label>,
    /// Unitary component's memory.
    pub memory: Memory,
}

impl PartialEq for ComponentDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.statements == other.statements
            && self.contract == other.contract
            && self.location == other.location
            && self.eq_graph(other)
    }
}

impl ComponentDefinition {
    /// Return vector of unitary node's signals id.
    pub fn get_signals_id(&self) -> Vec<usize> {
        self.statements
            .iter()
            .flat_map(|statement| statement.get_identifiers())
            .collect()
    }

    /// Return vector of unitary node's signals name.
    pub fn get_signals_names(&self, symbol_table: &SymbolTable) -> Vec<String> {
        self.statements
            .iter()
            .flat_map(|statement| statement.get_identifiers())
            .chain(self.memory.get_identifiers().cloned())
            .map(|id| symbol_table.get_name(id).clone())
            .collect()
    }

    fn eq_graph(&self, other: &ComponentDefinition) -> bool {
        let graph_nodes = self.graph.nodes();
        let other_nodes = other.graph.nodes();
        let graph_edges = self.graph.all_edges();
        let other_edges = other.graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }

    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.is_normal_form())
    }
    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.no_component_application())
    }
}

#[derive(Debug, Clone)]
/// LanGRust component import HIR.
pub struct ComponentImport {
    /// Component identifier.
    pub id: usize,
    /// Component path.
    pub path: syn::Path,
    /// Component location.
    pub location: Location,
    /// Component dependency graph.
    pub graph: DiGraphMap<usize, Label>,
}

impl PartialEq for ComponentImport {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.location == other.location && self.eq_graph(other)
    }
}

impl ComponentImport {
    fn eq_graph(&self, other: &ComponentImport) -> bool {
        let graph_nodes = self.graph.nodes();
        let other_nodes = other.graph.nodes();
        let graph_edges = self.graph.all_edges();
        let other_edges = other.graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }
}
