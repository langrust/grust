/// Structure of a vertex Neighbor.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Neighbor {
    /// The id of the neighbor.
    pub id: String,
    /// The label of the edge between the two vertices.
    pub label: Label,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Label {
    Contract,
    Weight(usize),
}

impl Neighbor {
    /// Creates a new neighbor.
    pub fn new(id: String, label: Label) -> Self {
        Neighbor { id, label }
    }
}

#[cfg(test)]
mod new {
    use crate::common::graph::neighbor::{Neighbor, Label};

    #[test]
    fn should_create_a_neighbor_with_corresponding_id_and_label() {
        let neighbor = Neighbor::new(String::from("v1"), Label::Weight(1));
        let control = Neighbor {
            id: String::from("v1"),
            label: Label::Weight(1),
        };

        assert_eq!(neighbor, control)
    }
}
