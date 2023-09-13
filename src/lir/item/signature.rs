use crate::lir::r#type::Type;

/// Function or method signature.
pub struct Signature {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Function or method's name.
    pub name: String,
    /// Optional `self` input.
    pub receiver: Option<Receiver>,
    /// List of inputs.
    pub inputs: Vec<(String, Type)>,
    /// Returned type.
    pub output: Type,
}

/// The `self` argument of a method.
pub struct Receiver {
    /// Reference: `true` is reference, `false` is owned.
    pub reference: bool,
    /// Mutability: `true` is mutable, `false` is immutable.
    pub mutable: bool,
}
