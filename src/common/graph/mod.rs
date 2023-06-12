/// [Vertex] structure and API.
pub mod vertex;

/// [Neighbor] structure and API.
pub mod neighbor;

use std::collections::HashMap;

use crate::common::graph::vertex::Vertex;

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
        self.has_vertex(from) && self.get_vertex(from).has_neighbor(to, weight)
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
