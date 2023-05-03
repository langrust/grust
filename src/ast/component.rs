use crate::util::location::Location;

#[derive(Debug, PartialEq)]
/// LanGRust component AST.
pub struct Component {
    /// Component location.
    pub location: Location,
}
