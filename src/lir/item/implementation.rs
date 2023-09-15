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

impl std::fmt::Display for Implementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let trait_name = if let Some(trait_name) = &self.trait_name {
            format!(" {trait_name} for")
        } else {
            "".to_string()
        };
        let items = self
            .items
            .iter()
            .map(|item| format!("{item}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "impl{} {} {{{}}}", trait_name, self.type_name, items)
    }
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

impl std::fmt::Display for AssociatedItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssociatedItem::AssociatedType { name, r#type } => {
                write!(f, "type {name} = {};", r#type)
            }
            AssociatedItem::AssociatedMethod { signature, body } => {
                write!(f, "{signature} {body}")
            }
        }
    }
}
