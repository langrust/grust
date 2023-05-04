use crate::util::{constant::Constant, location::Location};

#[derive(Debug, PartialEq)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Expression location.
        location: Location,
    },
    /// Call expression.
    Call {
        /// Element identifier.
        id: String,
        /// Expression location.
        location: Location,
    },
}
