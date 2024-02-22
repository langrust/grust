#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Edge label.
pub enum Label {
    /// Contract label
    Contract,
    /// Weighted label
    Weight(usize),
}

impl Label {
    pub fn add(&self, other: &Label) -> Label {
        match (self, other) {
            (Label::Contract, _) => Label::Contract,
            (_, Label::Contract) => Label::Contract,
            (Label::Weight(w1), Label::Weight(w2)) => Label::Weight(w1 + w2),
        }
    }
    pub fn increment(&self) -> Label {
        match self {
            Label::Contract => Label::Contract,
            Label::Weight(w) => Label::Weight(w + 1),
        }
    }
}
