use crate::lir::{block::Block, item::signature::Signature, r#type::Type};

/// Rust trait definition.
pub struct Trait {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// The name of the trait.
    pub trait_name: String,
    /// All the items of the trait.
    pub items: Vec<TraitAssociatedItem>,
}

/// Items that can be defined in a trait.
pub enum TraitAssociatedItem {
    /// Trait associated type definition.
    TraitAssociatedType {
        /// Associated type's name.
        name: String,
        /// Default value of the associated type.
        default: Option<Type>,
    },
    /// Trait associated method implementation.
    TraitAssociatedMethod {
        /// Method's signature.
        signature: Signature,
        /// Default body.
        default: Option<Block>,
    },
}
