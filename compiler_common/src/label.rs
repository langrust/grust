#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Edge label.
pub enum Label {
    /// Contract label.
    Contract,
    /// Weighted label.
    Weight(usize),
}

mk_new! { impl Label =>
    Contract: contract()
    Weight: weight(n: usize = n)
}

impl Label {
    /// Add the two given labels.
    pub fn add(&self, other: &Label) -> Label {
        match (self, other) {
            (Label::Contract, _) => Label::Contract,
            (_, Label::Contract) => Label::Contract,
            (Label::Weight(w1), Label::Weight(w2)) => Label::Weight(w1 + w2),
        }
    }
    /// Increment the given label.
    pub fn increment(&self) -> Label {
        match self {
            Label::Contract => Label::Contract,
            Label::Weight(w) => Label::Weight(w + 1),
        }
    }
}
