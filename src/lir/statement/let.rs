use crate::lir::expression::Expression;

/// A `let` binding: `let x: u64 = 5`.
pub struct Let {
    /// Reference: `true` is reference, `false` is owned.
    pub reference: bool,
    /// Mutability: `true` is mutable, `false` is immutable.
    pub mutable: bool,
    /// The created variable.
    pub identifiant: String,
    /// The associated expression.
    pub expression: Expression,
}
