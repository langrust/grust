/// [Color] enumeration used to identify the processing status of an element.
pub mod color;

/// [Vertex] structure and API.
pub mod vertex;

/// [Neighbor] structure and API.
pub mod neighbor;

/// [Color] enumeration used to identify the processing status of an element.
pub mod color;

use std::collections::HashMap;

use crate::common::{
    graph::{color::Color, neighbor::Label, vertex::Vertex},
    serialize::ordered_map,
};

use self::neighbor::Neighbor;

/// Graph structure.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Graph<T>
where
    T: serde::Serialize,
{
    /// Graph's vertices.
    #[serde(serialize_with = "ordered_map")]
    vertices: HashMap<String, Vertex<T>>,
}

impl<T> Graph<T>
where
    T: serde::Serialize,
{
    /// Creates a new graph with no vertices.
    pub fn new() -> Self {
        Graph {
            vertices: HashMap::new(),
        }
    }

    /// Add a vertex to the graph.
    pub fn add_vertex(&mut self, id: String, value: T) {
        if !self.has_vertex(&id) {
            self.vertices.insert(id.clone(), Vertex::new(id, value));
        }
    }

    /// Tells if the vertex is already in the graph
    pub fn has_vertex(&self, id: &String) -> bool {
        self.vertices.contains_key(id)
    }

    /// Get a vertex as reference.
    pub fn get_vertex(&self, id: &String) -> &Vertex<T> {
        self.vertices.get(id).unwrap()
    }

    /// Get a vertex as mutable reference.
    pub fn get_vertex_mut(&mut self, id: &String) -> &mut Vertex<T> {
        self.vertices.get_mut(id).unwrap()
    }

    /// Set vertex's value.
    pub fn set_vertex_value(&mut self, id: &String, value: T) {
        self.get_vertex_mut(id).set_value(value)
    }

    /// Add weighted edge between existing vertices to the graph.
    pub fn add_weighted_edge(&mut self, from: &String, to: String, weight: usize) {
        if !(self.has_vertex(from) && self.has_vertex(&to)) {
            panic!("vertices '{from}' or '{to}' do not exist")
        }
        if !self.has_weighted_edge(from, &to, &weight) {
            self.get_vertex_mut(from)
                .add_neighbor(to, Label::Weight(weight))
        }
    }

    /// Add edge between existing vertices to the graph.
    pub fn add_edge(&mut self, from: &String, to: String, label: Label) {
        if !(self.has_vertex(from) && self.has_vertex(&to)) {
            panic!("vertices '{from}' or '{to}' do not exist")
        }
        if !self.has_edge(from, &to, &label) {
            self.get_vertex_mut(from).add_neighbor(to, label)
        }
    }

    /// Tells if weighted edge already exist with this weight.
    pub fn has_weighted_edge(&self, from: &String, to: &String, weight: &usize) -> bool {
        self.has_vertex(from)
            && self
                .get_vertex(from)
                .has_neighbor_label(to, &Label::Weight(*weight))
    }

    /// Tells if edge already exist with this weight.
    pub fn has_edge(&self, from: &String, to: &String, label: &Label) -> bool {
        self.has_vertex(from) && self.get_vertex(from).has_neighbor_label(to, label)
    }

    /// Get vertices' ids sorted by key.
    pub fn get_vertices(&self) -> Vec<String> {
        let mut vertices = self.vertices.keys().cloned().collect::<Vec<_>>();
        vertices.sort();
        vertices
    }

    /// Get edges as pairs of ids.
    pub fn get_weighted_edges(&self) -> Vec<(String, String, usize)> {
        self.vertices
            .values()
            .flat_map(|vertex| {
                vertex
                    .neighbors
                    .iter()
                    .map(|neighbor| match neighbor.label {
                        Label::Contract => todo!(),
                        Label::Weight(weight) => (vertex.id.clone(), neighbor.id.clone(), weight),
                    })
                    .collect::<Vec<(String, String, usize)>>()
            })
            .collect::<Vec<(String, String, usize)>>()
    }

    /// Get weight of an edge if exists
    pub fn get_weights(&self, from: &String, to: &String) -> Vec<usize> {
        self.get_vertex(from).get_weights(to)
    }

    /// Create a copy of the graph without edges.
    pub fn no_edges_graph(&self) -> Graph<T>
    where
        T: Clone,
    {
        let mut subgraph = Graph::new();

        for vertex in self.vertices.values() {
            subgraph.add_vertex(vertex.id.clone(), vertex.get_value().clone())
        }

        subgraph
    }

    /// Create a subgraph from a predicate on edges' weights.
    pub fn subgraph_on_edges(&self, predicate: impl Fn(usize) -> bool) -> Graph<T>
    where
        T: Clone,
    {
        let mut subgraph = self.no_edges_graph();

        for (from, to, weight) in self.get_weighted_edges() {
            if predicate(weight) {
                subgraph.add_weighted_edge(&from, to, weight)
            }
        }

        subgraph
    }
}
impl<T> Default for Graph<T>
where
    T: serde::Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}
impl Graph<Color> {
    /// Topological sorting of an oriented graph.
    ///
    /// Scans an oriented graph and returns a schedule visiting all vertices in order.
    pub fn topological_sorting(&mut self) -> Result<Vec<String>, String> {
        // initialize schedule
        let mut schedule = vec![];

        // initialize all vertices to "unprocessed" state
        self.vertices
            .values_mut()
            .for_each(|vertex| vertex.set_value(Color::White));

        // process of vertices
        self.get_vertices()
            .iter()
            .try_for_each(|id| self.topological_sorting_visit(id, &mut schedule))?;

        Ok(schedule)
    }

    fn topological_sorting_visit(
        &mut self,
        id: &String,
        schedule: &mut Vec<String>,
    ) -> Result<(), String> {
        let vertex = self.get_vertex_mut(id);

        match vertex.get_value() {
            Color::White => {
                // update vertex status: processing
                vertex.set_value(Color::Grey);

                // processus propagation
                vertex
                    .get_neighbors()
                    .iter()
                    .try_for_each(|Neighbor { id, .. }| {
                        self.topological_sorting_visit(id, schedule)
                    })?;

                // update vertex status: processed
                let vertex = self.get_vertex_mut(id);
                vertex.set_value(Color::Black);

                // add vertex to schedule
                schedule.push(id.clone());

                Ok(())
            }
            Color::Grey => Err(id.clone()),
            Color::Black => Ok(()),
        }
    }

    /// Create a subgraph from the vertex.
    ///
    /// This creates a subgraph with all successors of the given vertex
    /// and their edges.
    pub fn subgraph_from_vertex(&self, vertex: &String, only_weight: bool) -> Graph<Color> {
        let mut graph = self.clone();

        // initialize subgraph
        let mut subgraph = Graph::new();

        // initialize all global graph vertices to "unprocessed" state
        graph
            .vertices
            .values_mut()
            .for_each(|vertex| vertex.set_value(Color::White));

        // process of vertices
        graph.subgraph_from_vertex_visit(vertex, only_weight, &mut subgraph);

        subgraph
    }

    fn subgraph_from_vertex_visit(
        &mut self,
        id: &String,
        only_weight: bool,
        subgraph: &mut Graph<Color>,
    ) {
        // add vertex to subgraph
        subgraph.add_vertex(id.clone(), Color::White);

        // visit vertex successors
        let vertex = self.get_vertex_mut(id);
        if &Color::White == vertex.get_value() {
            // update vertex status: processing
            vertex.set_value(Color::Grey);

            // processus propagation
            vertex.get_neighbors().iter().for_each(
                |Neighbor {
                     id: neighbor,
                     label,
                 }| {
                    match label {
                        Label::Contract if only_weight => (),
                        _ => {
                            // visit vertex successors
                            self.subgraph_from_vertex_visit(neighbor, only_weight, subgraph);
                            // add edge
                            subgraph.add_edge(id, neighbor.clone(), label.clone())
                        }
                    }
                },
            );

            // update vertex status: processed
            let vertex = self.get_vertex_mut(id);
            vertex.set_value(Color::Black);
        }
    }

    /// Returns mother graph forgotten vertices from subgraphs.
    ///
    /// Returns identifiers of vertices that do not appear in subgraphs set.
    pub fn forgotten_vertices(&self, subgraphs: Vec<Graph<Color>>) -> Vec<String> {
        let mut graph = self.clone();

        // initialize all global graph vertices to "unused" state
        graph
            .vertices
            .values_mut()
            .for_each(|vertex| vertex.set_value(Color::White));

        for subgraph in subgraphs {
            // in mother graph, set all vertices
            // appearing in subgraph to "used" state
            subgraph
                .get_vertices()
                .iter()
                .for_each(|vertex| graph.get_vertex_mut(vertex).set_value(Color::Black))
        }

        // returns vertices from mother graph in "unused" state
        graph
            .vertices
            .values()
            .filter(|vertex| vertex.get_value().eq(&Color::White))
            .map(|vertex| vertex.id.clone())
            .collect()
    }

    /// Tells if there is a loop of weighted edges from the given vertex.
    pub fn is_loop(&mut self, id: &String) -> bool {
        // initialize all vertices to "unprocessed" state
        self.vertices
            .values_mut()
            .for_each(|vertex| vertex.set_value(Color::White));

        // start visiting the graph
        self.is_loop_visit(id, id)
    }

    fn is_loop_visit(&mut self, id_start: &String, id_current: &String) -> bool {
        // visit vertex successors
        let vertex = self.get_vertex_mut(id_current);
        match vertex.get_value() {
            Color::White => {
                // update vertex status: processing
                vertex.set_value(Color::Grey);

                // processus propagation
                vertex.get_neighbors().iter().any(
                    |Neighbor {
                         id: neighbor,
                         label,
                     }| {
                        match label {
                            Label::Contract => false,
                            Label::Weight(_) => {
                                // visit vertex successors
                                self.is_loop_visit(id_start, neighbor)
                            }
                        }
                    },
                )
            }
            // if the vertex has been seen then check if we made a loop
            Color::Grey => id_start == id_current,
            Color::Black => unreachable!(),
        }
    }
}

#[cfg(test)]
mod new {
    use std::collections::HashMap;

    use crate::common::graph::Graph;

    #[test]
    fn should_create_empty_graph() {
        let graph: Graph<i32> = Graph::new();

        let control = Graph {
            vertices: HashMap::new(),
        };

        assert_eq!(graph, control)
    }
}

#[cfg(test)]
mod add_vertex {
    use std::collections::HashMap;

    use crate::common::graph::{vertex::Vertex, Graph};

    #[test]
    fn should_add_vertex_to_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);

        let control = Graph {
            vertices: HashMap::from([
                (String::from("v1"), Vertex::new(String::from("v1"), 1)),
                (String::from("v2"), Vertex::new(String::from("v2"), 2)),
            ]),
        };

        assert_eq!(graph, control)
    }

    #[test]
    fn should_not_duplicate_vertices_and_use_first_defined() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v1"), 2);

        let control = Graph {
            vertices: HashMap::from([(String::from("v1"), Vertex::new(String::from("v1"), 1))]),
        };

        assert_eq!(graph, control)
    }
}

#[cfg(test)]
mod has_vertex {
    use crate::common::graph::Graph;

    #[test]
    fn should_tell_when_vertex_is_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        assert!(graph.has_vertex(&String::from("v1")))
    }

    #[test]
    fn should_tell_when_vertex_is_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        assert!(!graph.has_vertex(&String::from("v2")))
    }
}

#[cfg(test)]
mod get_vertex {
    use crate::common::graph::{vertex::Vertex, Graph};

    #[test]
    fn should_get_vertex_when_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);

        let vertex = graph.get_vertex(&String::from("v1"));
        let control = Vertex::new(String::from("v1"), 1);

        assert_eq!(vertex, &control)
    }

    #[test]
    #[should_panic]
    fn should_panic_when_vertex_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        let _ = graph.get_vertex(&String::from("v2"));
    }
}

#[cfg(test)]
mod get_vertex_mut {
    use crate::common::graph::{vertex::Vertex, Graph};

    #[test]
    fn should_get_vertex_when_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);

        let vertex = graph.get_vertex_mut(&String::from("v1"));
        let control = Vertex::new(String::from("v1"), 1);

        assert_eq!(vertex, &control)
    }

    #[test]
    #[should_panic]
    fn should_panic_when_vertex_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        let _ = graph.get_vertex_mut(&String::from("v2"));
    }
}

#[cfg(test)]
mod set_vertex_value {
    use crate::common::graph::{vertex::Vertex, Graph};

    #[test]
    fn should_set_vertex_value_when_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.set_vertex_value(&String::from("v1"), 2);

        let vertex = graph.get_vertex(&String::from("v1"));
        let control = Vertex::new(String::from("v1"), 2);

        assert_eq!(vertex, &control)
    }

    #[test]
    #[should_panic]
    fn should_panic_when_vertex_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.set_vertex_value(&String::from("v2"), 2);
    }
}

#[cfg(test)]
mod add_weighted_edge {
    use std::collections::HashMap;

    use crate::common::graph::{neighbor::Label, vertex::Vertex, Graph};

    #[test]
    fn should_add_weighted_edge_between_existing_vertices() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);

        let mut v1 = Vertex::new(String::from("v1"), 1);
        let v2 = Vertex::new(String::from("v2"), 2);
        v1.add_neighbor(v2.id.clone(), Label::Weight(3));
        let control = Graph {
            vertices: HashMap::from([(String::from("v1"), v1), (String::from("v2"), v2)]),
        };

        assert_eq!(graph, control)
    }

    #[test]
    #[should_panic]
    fn should_panic_when_vertices_do_not_exist() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v1"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
    }
}

#[cfg(test)]
mod has_edge {
    use crate::common::graph::Graph;

    #[test]
    fn should_tell_when_edge_is_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        assert!(graph.has_weighted_edge(&String::from("v1"), &String::from("v2"), &3))
    }

    #[test]
    fn should_tell_when_edge_is_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        assert!(!graph.has_weighted_edge(&String::from("v1"), &String::from("v2"), &2))
    }

    #[test]
    fn should_not_panic_when_vertices_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v2"), 2);
        assert!(!graph.has_weighted_edge(&String::from("v1"), &String::from("v2"), &2))
    }
}

#[cfg(test)]
mod get_vertices {
    use crate::common::graph::Graph;

    #[test]
    fn should_get_vertices_ids() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);

        let mut vertices = graph.get_vertices();
        vertices.sort_unstable();

        let mut control = vec![String::from("v1"), String::from("v2")];
        control.sort_unstable();

        assert_eq!(vertices, control)
    }

    #[test]
    fn should_get_ids_in_key_order() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);

        let vertices = graph.get_vertices();

        for (i, vertex) in vertices.iter().enumerate() {
            if i > 0 {
                assert!(vertices.get(i - 1).unwrap() < vertex)
            }
        }
    }
}

#[cfg(test)]
mod get_weighted_edges {
    use crate::common::graph::Graph;

    #[test]
    fn should_get_all_edges() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 3);

        let mut edges = graph.get_weighted_edges();
        edges.sort_unstable();

        let mut control = vec![
            (String::from("v1"), String::from("v3"), 3),
            (String::from("v1"), String::from("v3"), 0),
            (String::from("v1"), String::from("v2"), 3),
        ];
        control.sort_unstable();

        assert_eq!(edges, control)
    }
}

#[cfg(test)]
mod get_weights {
    use crate::common::graph::Graph;

    #[test]
    fn should_get_vertex_neighbor_weight_in_optional_when_exists() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);

        let mut weights = graph.get_weights(&String::from("v1"), &String::from("v2"));
        weights.sort_unstable();
        let mut control = vec![0, 3];
        control.sort_unstable();

        assert_eq!(weights, control)
    }

    #[test]
    fn should_return_non_when_neighbor_does_not_exist() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);

        let control = vec![];

        assert_eq!(
            graph.get_weights(&String::from("v1"), &String::from("v3")),
            control
        )
    }
}

#[cfg(test)]
mod no_edges_graph {
    use crate::common::graph::Graph;

    #[test]
    fn should_return_graph_with_all_vertices() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 3);

        let subgraph = graph.no_edges_graph();

        let mut vertices = graph.get_vertices();
        vertices.sort_unstable();
        let mut subgraph_vertices = subgraph.get_vertices();
        subgraph_vertices.sort_unstable();

        assert_eq!(subgraph_vertices, vertices);
    }

    #[test]
    fn should_have_no_edges() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 3);

        let subgraph = graph.no_edges_graph();

        assert!(subgraph.get_weighted_edges().is_empty());
    }
}

#[cfg(test)]
mod subgraph_on_edges {
    use crate::common::graph::Graph;

    #[test]
    fn should_return_graph_with_all_vertices() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 3);

        let subgraph = graph.subgraph_on_edges(|weight| weight == 0);

        let mut vertices = graph.get_vertices();
        vertices.sort_unstable();
        let mut subgraph_vertices = subgraph.get_vertices();
        subgraph_vertices.sort_unstable();

        assert_eq!(subgraph_vertices, vertices);
    }

    #[test]
    fn should_have_edges_respecting_predicate() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 3);

        let subgraph = graph.subgraph_on_edges(|weight| weight == 0);

        let mut subgraph_edges = subgraph.get_weighted_edges();
        subgraph_edges.sort_unstable();

        let mut control = graph
            .get_weighted_edges()
            .into_iter()
            .filter(|(_, _, weight)| *weight == 0)
            .collect::<Vec<(String, String, usize)>>();
        control.sort_unstable();

        assert_eq!(subgraph_edges, control);
    }
}

#[cfg(test)]
mod topological_sorting {
    use crate::common::graph::{color::Color, Graph};

    #[test]
    fn should_return_a_schedule_of_the_graph_in_order() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v3"), String::from("v2"), 0);

        let schedule = graph.topological_sorting().unwrap();

        for (v1, v2, _) in graph.get_weighted_edges() {
            assert!(
                schedule.iter().position(|id| id.eq(&v1)).unwrap()
                    >= schedule.iter().position(|id| id.eq(&v2)).unwrap()
            );
        }
    }

    #[test]
    fn should_return_schedule_with_all_vertices() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v3"), String::from("v2"), 0);

        let schedule = graph.topological_sorting().unwrap();

        let vertices = graph.get_vertices();

        assert_eq!(schedule.len(), vertices.len());

        for vertex in vertices {
            assert!(schedule.iter().position(|id| id.eq(&vertex)).is_some())
        }
    }
}

#[cfg(test)]
mod subgraph_from_vertex {
    use crate::common::graph::{color::Color, Graph};

    #[test]
    fn should_return_a_subgraph_of_the_graph_with_all_vertex_successors_and_edges() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 1);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v3"), String::from("v2"), 0);

        let subgraph = graph.subgraph_from_vertex(&String::from("v1"), true);

        let mut control = Graph::new();
        control.add_vertex(String::from("v1"), Color::White);
        control.add_vertex(String::from("v2"), Color::White);
        control.add_vertex(String::from("v3"), Color::White);
        control.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        control.add_weighted_edge(&String::from("v1"), String::from("v2"), 1);
        control.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        control.add_weighted_edge(&String::from("v3"), String::from("v2"), 0);

        assert_eq!(subgraph, control)
    }
}

#[cfg(test)]
mod forgotten_vertices {
    use crate::common::graph::{color::Color, Graph};

    #[test]
    fn should_return_all_forgotten_vertices_of_subgraphs_set() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 1);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v3"), String::from("v2"), 0);

        let subgraph1 = graph.subgraph_from_vertex(&String::from("v1"), true);
        let subgraph2 = graph.subgraph_from_vertex(&String::from("v2"), true);
        let forgotten_vertices = graph.forgotten_vertices(vec![subgraph1, subgraph2]);

        let control = vec![String::from("v4")];

        assert_eq!(forgotten_vertices, control)
    }
}

#[cfg(test)]
mod is_loop {
    use crate::common::graph::{color::Color, Graph};

    #[test]
    fn should_return_true_if_there_is_a_loop() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v2"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v3"), String::from("v1"), 1);

        assert!(graph.is_loop(&String::from("v1")))
    }

    #[test]
    fn should_return_false_if_there_is_no_loop() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_weighted_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_weighted_edge(&String::from("v2"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_weighted_edge(&String::from("v3"), String::from("v2"), 1);

        assert!(!graph.is_loop(&String::from("v1")))
    }
}
