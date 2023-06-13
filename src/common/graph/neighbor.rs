/// Structure of a vertex Neighbor.
#[derive(Debug, PartialEq, Clone)]
pub struct Neighbor {
    /// The id of the neighbor.
    pub id: String,
    /// The weight of the edge between the two vertices.
    pub weight: usize,
}

impl Neighbor {
    /// Creates a new neighbor.
    pub fn new(id: String, weight: usize) -> Self {
        Neighbor { id, weight }
    }
}

#[cfg(test)]
mod new {
    use crate::common::graph::neighbor::Neighbor;

    #[test]
    fn should_create_a_neighbor_with_corresponding_id_and_weight() {
        let neighbor = Neighbor::new(String::from("v1"), 1);
        let control = Neighbor {
            id: String::from("v1"),
            weight: 1,
        };

        assert_eq!(neighbor, control)
    }
}
