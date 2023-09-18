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

impl std::fmt::Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        let items = self
            .items
            .iter()
            .map(|item| format!("{item}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{}trait {} {{ {} }}", visibility, self.trait_name, items)
    }
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

impl std::fmt::Display for TraitAssociatedItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitAssociatedItem::TraitAssociatedType { name, default } => {
                let default = if let Some(default) = default {
                    format!(" = {default}")
                } else {
                    "".to_string()
                };
                write!(f, "type {name}{default};")
            }
            TraitAssociatedItem::TraitAssociatedMethod { signature, default } => {
                let default = if let Some(default) = default {
                    format!(" {default}")
                } else {
                    ";".to_string()
                };
                write!(f, "{signature}{default}")
            }
        }
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::r#type::Type as DSLType,
        lir::{
            item::signature::{Receiver, Signature},
            r#type::Type,
        },
    };

    use super::{Trait, TraitAssociatedItem};

    #[test]
    fn should_format_trait_definition() {
        let r#trait = Trait {
            public_visibility: true,
            trait_name: String::from("Display"),
            items: vec![
                TraitAssociatedItem::TraitAssociatedType {
                    name: String::from("MyString"),
                    default: None,
                },
                TraitAssociatedItem::TraitAssociatedMethod {
                    signature: Signature {
                        public_visibility: false,
                        name: String::from("fmt"),
                        receiver: Some(Receiver {
                            reference: true,
                            mutable: false,
                        }),
                        inputs: vec![(String::from("f"), Type::MutableReference(DSLType::String))],
                        output: Type::Owned(DSLType::Unit),
                    },
                    default: None,
                },
            ],
        };
        let control =
            String::from("pub trait Display { type MyString; fn fmt(&self, f: &mut String); }");
        assert_eq!(format!("{}", r#trait), control)
    }
}
