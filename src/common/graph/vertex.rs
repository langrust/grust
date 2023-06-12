use crate::common::graph::neighbor::Neighbor;

/// Vertex structure for graph.
#[derive(Debug, PartialEq)]
pub struct Vertex<T> {
    /// Name fo the vertex.
    pub id: String,
    /// Value of the vertex.
    pub value: T,
    /// Vertex's neighbors.
    pub neighbors: Vec<Neighbor>,
}

impl<T> Vertex<T> {
    /// Creates a new vertex, with no neighbors.
    pub fn new(id: String, value: T) -> Self {
        Vertex {
            id,
            value,
            neighbors: vec![],
        }
    }

    /// Add a neighbor to the vertex.
    pub fn add_neighbor(&mut self, id: String, weight: usize) {
        if !self.has_neighbor(&id, &weight) {
            self.neighbors.push(Neighbor::new(id, weight))
        }
    }

    /// Tells if the neighbor is already known with this weight.
    pub fn has_neighbor(&self, id: &String, weight: &usize) -> bool {
        for Neighbor {
            id: other_id,
            weight: other_weight,
        } in &self.neighbors
        {
            if other_id.eq(id) && other_weight.eq(weight) {
                return true;
            }
        }
        return false;
    }

    /// Set a new value to the vertex.
    pub fn set_value(&mut self, value: T) {
        self.value = value;
    }

    /// Get the vertex's value.
    pub fn get_value(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod new {
    use crate::common::graph::vertex::Vertex;

    #[test]
    fn should_create_a_vertex_with_the_value_and_no_neighbors() {
        let vertex = Vertex::new(String::from("v1"), 1);
        let control = Vertex {
            id: String::from("v1"),
            value: 1,
            neighbors: vec![],
        };

        assert_eq!(vertex, control)
    }
}

#[cfg(test)]
mod add_neighbor {
    use crate::common::graph::vertex::Neighbor;

    use crate::common::graph::vertex::Vertex;

    #[test]
    fn should_add_neighbor_to_vertex() {
        let mut vertex = Vertex::new(String::from("v1"), 1);
        vertex.add_neighbor(String::from("v2"), 2);

        let control = Vertex {
            id: String::from("v1"),
            value: 1,
            neighbors: vec![Neighbor::new(String::from("v2"), 2)],
        };

        assert_eq!(vertex, control)
    }

    #[test]
    fn should_not_duplicate_neighbors() {
        let mut vertex = Vertex::new(String::from("v1"), 1);
        vertex.add_neighbor(String::from("v2"), 2);
        vertex.add_neighbor(String::from("v2"), 2);

        let control = Vertex {
            id: String::from("v1"),
            value: 1,
            neighbors: vec![Neighbor::new(String::from("v2"), 2)],
        };

        assert_eq!(vertex, control)
    }
}

#[cfg(test)]
mod has_neighbor {
    use crate::common::graph::vertex::Vertex;

    #[test]
    fn should_tell_when_vertex_has_neighbor() {
        let mut vertex = Vertex::new(String::from("v1"), 1);
        vertex.add_neighbor(String::from("v2"), 2);
        assert!(vertex.has_neighbor(&String::from("v2"), &2))
    }

    #[test]
    fn should_tell_when_vertex_does_not_have_neighbor() {
        let mut vertex = Vertex::new(String::from("v1"), 1);
        vertex.add_neighbor(String::from("v2"), 2);
        assert!(!vertex.has_neighbor(&String::from("v3"), &2))
    }
}

#[cfg(test)]
mod set_value {
    use crate::common::graph::vertex::Vertex;

    #[test]
    fn should_update_vertex_value() {
        let mut vertex = Vertex::new(String::from("v1"), 1);
        vertex.set_value(2);

        let control = Vertex {
            id: String::from("v1"),
            value: 2,
            neighbors: vec![],
        };

        assert_eq!(vertex, control)
    }
}

#[cfg(test)]
mod get_value {
    use crate::common::graph::vertex::Vertex;

    #[test]
    fn should_get_vertex_value() {
        let vertex = Vertex::new(String::from("v1"), 1);
        assert_eq!(vertex.get_value(), &1)
    }
}
