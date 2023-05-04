use crate::util::{location::Location, constant::Constant};

#[derive(Debug, PartialEq)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant{
        /// The constant.
        constant: Constant,
        /// Expression location.
        location: Location,
    },
}
