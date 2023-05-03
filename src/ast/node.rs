use crate::util::location::Location;

#[derive(Debug, PartialEq)]
/// LanGRust node AST.
pub struct Node {
    /// Node location.
    pub location: Location,
}
