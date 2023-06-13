/// [Vertex] structure and API.
pub mod vertex;

/// [Neighbor] structure and API.
pub mod neighbor;

use std::collections::HashMap;

use crate::{
    common::{color::Color, graph::vertex::Vertex},
    error::Error,
};

/// Graph structure.
#[derive(Debug, PartialEq)]
pub struct Graph<T> {
    /// Graph's vertices.
    vertices: HashMap<String, Vertex<T>>,
}

impl<T> Graph<T> {
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
        self.get_vertex_mut(&id).set_value(value)
    }

    /// Add edge between existing vertices to the graph.
    pub fn add_edge(&mut self, from: &String, to: String, weight: usize) {
        if !(self.has_vertex(from) && self.has_vertex(&to)) {
            panic!("vertices do not exist")
        }
        if !self.has_edge(from, &to, &weight) {
            self.get_vertex_mut(from).add_neighbor(to, weight)
        }
    }

    /// Tells if edge already exist with this weight.
    pub fn has_edge(&self, from: &String, to: &String, weight: &usize) -> bool {
        self.has_vertex(from) && self.get_vertex(from).has_neighbor_weight(to, weight)
    }

    /// Get vertices' ids.
    pub fn get_vertices(&self) -> Vec<String> {
        self.vertices.keys().map(|id| id.clone()).collect()
    }

    /// Get edges as pairs of ids, no duplicates.
    pub fn get_edges(&self) -> Vec<(String, String, usize)> {
        self.vertices
            .values()
            .flat_map(|vertex| {
                vertex
                    .neighbors
                    .iter()
                    .map(|neighbor| (vertex.id.clone(), neighbor.id.clone(), neighbor.weight))
                    .collect::<Vec<(String, String, usize)>>()
            })
            .collect::<Vec<(String, String, usize)>>()
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

        for (from, to, weight) in self.get_edges() {
            if predicate(weight) {
                subgraph.add_edge(&from, to, weight)
            }
        }

        subgraph
    }
}

impl Graph<Color> {
    /// Topological sorting of an oriented graph.
    ///
    /// Scans an oriented graph and returns a schedule visiting all vertices in order.
    pub fn topological_sorting(&mut self, errors: &mut Vec<Error>) -> Result<Vec<String>, ()> {
        // initialize schedule
        let mut schedule = vec![];

        // initialize all vertices to "unprocessed" state
        self.vertices
            .values_mut()
            .for_each(|vertex| vertex.set_value(Color::White));

        // process of vertices
        self.get_vertices()
            .iter()
            .map(|id| self.visit_vertex(&id, &mut schedule, errors))
            .collect::<Result<(), ()>>()?;

        Ok(schedule)
    }

    fn visit_vertex(
        &mut self,
        id: &String,
        schedule: &mut Vec<String>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let vertex = self.get_vertex_mut(id);

        match vertex.get_value() {
            Color::White => {
                // update vertex status: processing
                vertex.set_value(Color::Grey);

                // processus propagation
                vertex
                    .get_neighbors()
                    .iter()
                    .map(|id| self.visit_vertex(id, schedule, errors))
                    .collect::<Result<(), ()>>()?;

                // update vertex status: processed
                let vertex = self.get_vertex_mut(id);
                vertex.set_value(Color::Black);

                // add vertex to schedule
                schedule.insert(0, id.clone());

                Ok(())
            }
            Color::Grey => todo!("error"),
            Color::Black => Ok(()),
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
mod add_edge {
    use std::collections::HashMap;

    use crate::common::graph::{vertex::Vertex, Graph};

    #[test]
    fn should_add_edge_between_existing_vertices() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);

        let mut v1 = Vertex::new(String::from("v1"), 1);
        let v2 = Vertex::new(String::from("v2"), 2);
        v1.add_neighbor(v2.id.clone(), 3);
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
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
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
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
        assert!(graph.has_edge(&String::from("v1"), &String::from("v2"), &3))
    }

    #[test]
    fn should_tell_when_edge_is_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        assert!(!graph.has_edge(&String::from("v1"), &String::from("v2"), &2))
    }

    #[test]
    fn should_not_panic_when_vertices_not_in_graph() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v2"), 2);
        assert!(!graph.has_edge(&String::from("v1"), &String::from("v2"), &2))
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
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);

        let mut vertices = graph.get_vertices();
        vertices.sort_unstable();

        let mut control = vec![String::from("v1"), String::from("v2")];
        control.sort_unstable();

        assert_eq!(vertices, control)
    }
}

#[cfg(test)]
mod get_edges {
    use crate::common::graph::Graph;

    #[test]
    fn should_get_all_edges() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 3);

        let mut edges = graph.get_edges();
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
mod no_edges_graph {
    use crate::common::graph::Graph;

    #[test]
    fn should_return_graph_with_all_vertices() {
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), 1);
        graph.add_vertex(String::from("v2"), 2);
        graph.add_vertex(String::from("v3"), 2);
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 3);

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
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 3);

        let subgraph = graph.no_edges_graph();

        assert!(subgraph.get_edges().is_empty());
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
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 3);

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
        graph.add_edge(&String::from("v1"), String::from("v2"), 3);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 3);

        let subgraph = graph.subgraph_on_edges(|weight| weight == 0);

        let mut subgraph_edges = subgraph.get_edges();
        subgraph_edges.sort_unstable();

        let mut control = graph
            .get_edges()
            .into_iter()
            .filter(|(_, _, weight)| *weight == 0)
            .collect::<Vec<(String, String, usize)>>();
        control.sort_unstable();

        assert_eq!(subgraph_edges, control);
    }
}

#[cfg(test)]
mod topological_sorting {
    use crate::common::{color::Color, graph::Graph};

    #[test]
    fn should_return_a_schedule_of_the_graph_in_order() {
        let mut errors = vec![];

        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v3"), String::from("v2"), 0);

        let schedule = graph.topological_sorting(&mut errors).unwrap();

        for (v1, v2, _) in graph.get_edges() {
            assert!(
                schedule.iter().position(|id| id.eq(&v1)).unwrap()
                    <= schedule.iter().position(|id| id.eq(&v2)).unwrap()
            );
        }
    }

    #[test]
    fn should_return_schedule_with_all_vertices() {
        let mut errors = vec![];

        let mut graph = Graph::new();
        graph.add_vertex(String::from("v1"), Color::Black);
        graph.add_vertex(String::from("v2"), Color::Black);
        graph.add_vertex(String::from("v3"), Color::Black);
        graph.add_vertex(String::from("v4"), Color::Black);
        graph.add_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_edge(&String::from("v1"), String::from("v2"), 0);
        graph.add_edge(&String::from("v1"), String::from("v3"), 0);
        graph.add_edge(&String::from("v3"), String::from("v2"), 0);

        let schedule = graph.topological_sorting(&mut errors).unwrap();

        let vertices = graph.get_vertices();

        assert_eq!(schedule.len(), vertices.len());

        for vertex in vertices {
            assert!(schedule.iter().position(|id| id.eq(&vertex)).is_some())
        }
    }
}
