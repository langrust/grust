#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Edge label.
pub enum Label {
    /// Contract label
    Contract,
    /// Weighted label
    Weight(usize),
}
