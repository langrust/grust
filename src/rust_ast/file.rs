use super::item::Item;

#[derive(serde::Serialize)]
/// HIR of a Rust source code file.
pub struct File {
    /// File's path.
    pub path: String,
    /// Items present in the file.
    pub items: Vec<Item>,
}
impl File {
    /// Create a new file.
    pub fn new(path: String) -> Self {
        File {
            path,
            items: vec![],
        }
    }

    /// Add item.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item)
    }

    /// Generate the file at its location path.
    pub fn generate(&self) {
        let file_str = self.to_string();
        let syntax_tree: syn::File = syn::parse_str(&file_str).unwrap();
        let pretty_file = prettyplease::unparse(&syntax_tree);

        if let Some(p) = AsRef::<std::path::Path>::as_ref(&self.path).parent() {
            std::fs::create_dir_all(p).unwrap()
        };
        std::fs::write(self.path.clone(), pretty_file).unwrap();
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self
            .items
            .iter()
            .map(|item| format!("{item}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{items}")
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{constant::Constant, operator::BinaryOperator},
        rust_ast::{
            block::Block,
            expression::Expression,
            file::File,
            item::Item,
            item::{
                enumeration::Enumeration,
                function::Function,
                implementation::{AssociatedItem, Implementation},
                import::{Import, PathTree},
                r#trait::{Trait, TraitAssociatedItem},
                signature::{Receiver, Signature},
                structure::{Field, Structure},
                type_alias::TypeAlias,
            },
            pattern::Pattern,
            r#type::Type,
            statement::{r#let::Let, Statement},
        },
    };

    #[test]
    fn should_format_file() {
        let enumeration = Item::Enumeration(Enumeration {
            public_visibility: true,
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Yellow"),
                String::from("Green"),
                String::from("Purple"),
            ],
        });
        let function = Item::Function(Function {
            signature: Signature {
                public_visibility: true,
                name: String::from("foo"),
                receiver: None,
                inputs: vec![
                    (
                        String::from("x"),
                        Type::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                    (
                        String::from("y"),
                        Type::Identifier {
                            identifier: String::from("i64"),
                        },
                    ),
                ],
                output: Type::Identifier {
                    identifier: String::from("i64"),
                },
            },
            body: Block {
                statements: vec![
                    Statement::Let(Let {
                        pattern: Pattern::Identifier {
                            reference: false,
                            mutable: true,
                            identifier: String::from("z"),
                        },
                        expression: Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("x"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Identifier {
                                identifier: String::from("y"),
                            }),
                        },
                    }),
                    Statement::ExpressionIntern(Expression::Assignement {
                        left: Box::new(Expression::Identifier {
                            identifier: String::from("z"),
                        }),
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Identifier {
                                identifier: String::from("z"),
                            }),
                            operator: BinaryOperator::Add,
                            right: Box::new(Expression::Literal {
                                literal: Constant::Integer(1),
                            }),
                        }),
                    }),
                    Statement::ExpressionLast(Expression::Identifier {
                        identifier: String::from("z"),
                    }),
                ],
            },
        });
        let trait_impl = Item::Implementation(Implementation {
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
        });
        let use_import1 = Item::Import(Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("fmt"),
                    tree: Box::new(PathTree::Name {
                        name: String::from("Debug"),
                        alias: None,
                    }),
                }),
            },
        });
        let use_import2 = Item::Import(Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("future"),
                    tree: Box::new(PathTree::Name {
                        name: String::from("Future"),
                        alias: Some(String::from("AliasFuture")),
                    }),
                }),
            },
        });
        let use_import3 = Item::Import(Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("sync"),
                    tree: Box::new(PathTree::Star),
                }),
            },
        });
        let use_import4 = Item::Import(Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Group {
                    trees: vec![
                        PathTree::Path {
                            module_name: String::from("sync"),
                            tree: Box::new(PathTree::Star),
                        },
                        PathTree::Path {
                            module_name: String::from("fmt"),
                            tree: Box::new(PathTree::Name {
                                name: String::from("Debug"),
                                alias: None,
                            }),
                        },
                        PathTree::Path {
                            module_name: String::from("future"),
                            tree: Box::new(PathTree::Name {
                                name: String::from("Future"),
                                alias: Some(String::from("AliasFuture")),
                            }),
                        },
                    ],
                }),
            },
        });
        let mod_import = Item::Import(Import::Module {
            public_visibility: true,
            name: String::from("my_module"),
        });
        let structure = Item::Structure(Structure {
            public_visibility: true,
            name: String::from("Point"),
            fields: vec![
                Field {
                    public_visibility: true,
                    name: String::from("x"),
                    r#type: Type::Identifier {
                        identifier: String::from("i64"),
                    },
                },
                Field {
                    public_visibility: true,
                    name: String::from("y"),
                    r#type: Type::Identifier {
                        identifier: String::from("i64"),
                    },
                },
                Field {
                    public_visibility: false,
                    name: String::from("z"),
                    r#type: Type::Identifier {
                        identifier: String::from("i64"),
                    },
                },
            ],
        });
        let r#trait = Item::Trait(Trait {
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
                    default: None,
                },
            ],
        });
        let alias = Item::TypeAlias(TypeAlias {
            public_visibility: true,
            name: String::from("Integer"),
            r#type: Type::Identifier {
                identifier: String::from("i64"),
            },
        });
        let file = File {
            path: format!("my_file.rs"),
            items: vec![
                enumeration,
                function,
                trait_impl,
                use_import1,
                use_import2,
                use_import3,
                use_import4,
                mod_import,
                structure,
                r#trait,
                alias,
            ],
        };
        let control = String::from("pub enum Color { Blue, Red, Yellow, Green, Purple } ")
            + "pub fn foo(x: i64, y: i64) -> i64 { let mut z = x + y; z = z + 1i64; z } "
            + "impl Display for Point { type MyString = String; "
            + "fn fmt(&self, f: &mut String) { write!(f, \"({}, {})\", self.x, self.y); } } "
            + "pub use std::fmt::Debug; "
            + "pub use std::future::Future as AliasFuture; "
            + "pub use std::sync::*; "
            + "pub use std::{ sync::*, fmt::Debug, future::Future as AliasFuture }; "
            + "pub mod my_module; "
            + "pub struct Point { pub x: i64, pub y: i64, z: i64 } "
            + "pub trait Display { type MyString; fn fmt(&self, f: &mut String); } "
            + "pub type Integer = i64;";
        assert_eq!(format!("{}", file), control)
    }
}

