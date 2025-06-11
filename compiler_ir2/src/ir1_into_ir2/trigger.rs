use itertools::Itertools;

prelude! {
    graph::{DiGraphMap, DfsEvent::*}, HashSet,
    ir1::{ Service, interface::{ FlowImport, FlowStatement } },
}

use super::isles;

/// Graph of triggers.
pub trait TriggersGraph<'a> {
    fn new(ctx: &'a Ctx, service: &'a Service, imports: &'a HashMap<usize, FlowImport>) -> Self;
    fn get_triggered(&self, parent: usize) -> Vec<usize>;
    fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()>;
    fn graph(&self) -> DiGraphMap<usize, ()>;
}

/// Enumerate all the implementations of TriggersGraph.
pub enum Graph<'a> {
    EventIsles(EventIslesGraph<'a>),
    OnChange(OnChangeGraph<'a>),
}
impl<'a> TriggersGraph<'a> for Graph<'a> {
    fn new(ctx: &'a Ctx, service: &'a Service, imports: &'a HashMap<usize, FlowImport>) -> Self {
        match ctx.conf.propagation {
            conf::Propagation::EventIsles => {
                Graph::EventIsles(EventIslesGraph::new(ctx, service, imports))
            }
            conf::Propagation::OnChange => {
                Graph::OnChange(OnChangeGraph::new(ctx, service, imports))
            }
        }
    }

    fn get_triggered(&self, parent: usize) -> Vec<usize> {
        match self {
            Graph::EventIsles(graph) => graph.get_triggered(parent),
            Graph::OnChange(graph) => graph.get_triggered(parent),
        }
    }

    fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()> {
        match self {
            Graph::EventIsles(graph) => graph.subgraph(starts),
            Graph::OnChange(graph) => graph.subgraph(starts),
        }
    }

    fn graph(&self) -> DiGraphMap<usize, ()> {
        match self {
            Graph::EventIsles(graph) => graph.graph(),
            Graph::OnChange(graph) => graph.graph(),
        }
    }
}

/// Isles of statements triggered by events only.
pub struct EventIslesGraph<'a> {
    service: &'a Service,
    ctx: &'a Ctx,
    graph: &'a DiGraphMap<usize, ()>,
    stmts: &'a HashMap<usize, FlowStatement>,
    imports: &'a HashMap<usize, FlowImport>,
    isles: isles::Isles,
}
impl<'a> EventIslesGraph<'a> {
    /// Returns the identifiers of flows that are defined by the statement.
    fn get_def_flows(&self, id: usize) -> Vec<usize> {
        if let Some(stmt) = self.stmts.get(&id) {
            stmt.get_identifiers()
        } else if let Some(import) = self.imports.get(&id) {
            vec![import.id]
        } else {
            vec![]
        }
    }
    /// Tells if the statements is a component call.
    fn is_comp_call(&self, id: usize) -> bool {
        self.stmts
            .get(&id)
            .map_or(false, FlowStatement::is_comp_call)
    }

    /// Adds the directed dependencies between 'node' and other existing nodes of the
    /// `trig_graph`.
    fn add_nodes_deps(&self, trig_graph: &mut DiGraphMap<usize, ()>, node: usize) {
        for (from, to, ()) in self.graph.edges_directed(node, graph::Direction::Incoming) {
            debug_assert_eq!(node, to);
            if trig_graph.contains_node(from) {
                trig_graph.add_edge(from, to, ());
            }
        }
        for (from, to, ()) in self.graph.edges_directed(node, graph::Direction::Outgoing) {
            debug_assert_eq!(node, from);
            if trig_graph.contains_node(to) {
                trig_graph.add_edge(from, to, ());
            }
        }
    }
}
impl<'a> TriggersGraph<'a> for EventIslesGraph<'a> {
    fn new(ctx: &'a Ctx, service: &'a Service, imports: &'a HashMap<usize, FlowImport>) -> Self {
        // create events isles
        let mut isle_builder = isles::IsleBuilder::new(ctx, service, imports);
        isle_builder.trace_events(service.get_flows_ids(imports.values()));
        let isles = isle_builder.into_isles();

        EventIslesGraph {
            graph: &service.graph,
            stmts: &service.statements,
            imports,
            isles,
            ctx,
            service,
        }
    }

    fn get_triggered(&self, parent: usize) -> Vec<usize> {
        // if service timeout then trigger
        if self
            .get_def_flows(parent)
            .into_iter()
            .any(|id| self.ctx.is_service_timeout(self.service.id, id))
        {
            return self
                .service
                .statements
                .iter()
                .filter_map(|(stmt_id, stmt)| {
                    if stmt.is_comp_call() {
                        Some(*stmt_id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
        }

        // get graph dependencies
        let dependencies = self.graph.neighbors(parent).filter_map(|child| {
            // filter component call because they will appear in isles
            if self.is_comp_call(child) {
                return None;
            }
            Some(child)
        });

        // get isles dependencies
        let isles = self
            .get_def_flows(parent)
            .into_iter()
            .filter_map(|parent_flow| self.isles.get_isle_for(parent_flow))
            .flatten().copied();

        // extend stack with union of event isle and dependencies
        isles.chain(dependencies).unique().collect()
    }

    fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()> {
        let mut trig_graph = DiGraphMap::new();
        // init stack and seen set
        let (mut stack, mut seen) = (vec![], HashSet::new());
        starts.for_each(|id| {
            trig_graph.add_node(id);
            stack.push(id);
            seen.insert(id);
        });
        // loop on stack
        while let Some(parent) = stack.pop() {
            // add edges of dependency graph between existing nodes
            self.add_nodes_deps(&mut trig_graph, parent);

            // add triggered nodes
            let neighbors = self.get_triggered(parent);
            for child in neighbors {
                trig_graph.add_edge(parent, child, ());
                // only insert in stack if not seen
                if seen.insert(child) {
                    stack.push(child);
                }
            }
        }
        trig_graph
    }

    fn graph(&self) -> DiGraphMap<usize, ()> {
        self.subgraph(self.graph.nodes())
    }
}

/// Statements triggered by all changes.
pub struct OnChangeGraph<'a> {
    graph: &'a DiGraphMap<usize, ()>,
}
impl<'a> TriggersGraph<'a> for OnChangeGraph<'a> {
    fn new(_ctx: &'a Ctx, service: &'a Service, _imports: &'a HashMap<usize, FlowImport>) -> Self {
        OnChangeGraph {
            graph: &service.graph,
        }
    }

    fn get_triggered(&self, parent: usize) -> Vec<usize> {
        // get graph dependencies
        self.graph.neighbors(parent).collect()
    }

    fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()> {
        let mut trig_graph = DiGraphMap::new();
        let starts = starts.collect::<Vec<_>>();
        starts.iter().for_each(|id| {
            trig_graph.add_node(*id);
        });
        graph::visit::depth_first_search(&self.graph, starts, |event| match event {
            CrossForwardEdge(parent, child) | BackEdge(parent, child) | TreeEdge(parent, child) => {
                // add in subgraph of triggers
                trig_graph.add_edge(parent, child, ());
            }
            Discover(_, _) | Finish(_, _) => {}
        });
        trig_graph
    }

    fn graph(&self) -> DiGraphMap<usize, ()> {
        self.graph.clone()
    }
}
