use crate::lir::{block::Block, item::signature::Signature, r#type::Type};

/// Rust trait or type implementation.
pub struct Implementation {
    /// The optional trait that might is implemented.
    pub trait_name: Option<String>,
    /// The implemented type.
    pub type_name: String,
    /// All the items of the implementation.
    pub items: Vec<AssociatedItem>,
}

/// Items that can be defined in an implementation.
pub enum AssociatedItem {
    /// Associated type definition.
    AssociatedType {
        /// Associated type's name.
        name: String,
        /// The Rust type of the associated type.
        r#type: Type,
    },
    /// Associated method implementation.
    AssociatedMethod {
        /// Method's signature.
        signature: Signature,
        /// Method's body.
        body: Block,
    },
}
