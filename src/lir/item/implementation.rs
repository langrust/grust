use crate::lir::{block::Block, item::signature::Signature, r#type::Type};

/// Rust trait or type implementation.
#[derive(Debug, PartialEq, serde::Serialize)]
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
        write!(f, "impl{} {} {{ {} }}", trait_name, self.type_name, items)
    }
}

/// Items that can be defined in an implementation.
#[derive(Debug, PartialEq, serde::Serialize)]
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

#[cfg(test)]
mod fmt {
    use crate::{
        common::constant::Constant,
        lir::{
            block::Block,
            expression::Expression,
            item::{
                implementation::{AssociatedItem, Implementation},
                signature::{Receiver, Signature},
            },
            r#type::Type,
            statement::Statement,
        },
    };

    #[test]
    fn should_format_trait_implementation() {
        let r#trait = Implementation {
            trait_name: Some(String::from("Display")),
            type_name: String::from("Point"),
            items: vec![
                AssociatedItem::AssociatedType {
                    name: String::from("MyString"),
                    r#type: Type::Identifier {
                        identifier: String::from("String"),
                    },
                },
                AssociatedItem::AssociatedMethod {
                    signature: Signature {
                        public_visibility: false,
                        name: String::from("fmt"),
                        receiver: Some(Receiver {
                            reference: true,
                            mutable: false,
                        }),
                        inputs: vec![(
                            String::from("f"),
                            Type::Reference {
                                mutable: true,
                                element: Box::new(Type::Identifier {
                                    identifier: String::from("String"),
                                }),
                            },
                        )],
                        output: Type::Identifier {
                            identifier: String::from("()"),
                        },
                    },
                    body: Block {
                        statements: vec![Statement::ExpressionIntern(Expression::Macro {
                            r#macro: String::from("write"),
                            arguments: vec![
                                Expression::Identifier {
                                    identifier: String::from("f"),
                                },
                                Expression::Literal {
                                    literal: Constant::String(String::from("({}, {})")),
                                },
                                Expression::FieldAccess {
                                    expression: Box::new(Expression::Identifier {
                                        identifier: String::from("self"),
                                    }),
                                    field: String::from("x"),
                                },
                                Expression::FieldAccess {
                                    expression: Box::new(Expression::Identifier {
                                        identifier: String::from("self"),
                                    }),
                                    field: String::from("y"),
                                },
                            ],
                        })],
                    },
                },
            ],
        };
        let control = String::from("impl Display for Point { type MyString = String; ")
            + "fn fmt(&self, f: &mut String) { write!(f, \"({}, {})\", self.x, self.y); } }";
        assert_eq!(format!("{}", r#trait), control)
    }
}
