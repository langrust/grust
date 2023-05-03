use crate::util::location::Location;

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function location.
    pub location: Location,
}
