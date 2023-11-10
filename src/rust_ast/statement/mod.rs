use crate::rust_ast::{expression::Expression, item::Item, statement::r#let::Let};

/// LIR [Let](crate::lir::statement::r#let::Let) module.
pub mod r#let;

/// Rust statement.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum Statement {
    /// A `let` binding.
    Let(Let),
    /// An item definition.
    Item(Item),
    /// An internal expression, endding with a semicolon.
    ExpressionIntern(Expression),
    /// The last expression, no semicolon.
    ExpressionLast(Expression),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(r#let) => write!(f, "{}", r#let),
            Statement::Item(item) => write!(f, "{item}"),
            Statement::ExpressionIntern(expression) => write!(f, "{expression};"),
            Statement::ExpressionLast(expression) => write!(f, "{expression}"),
        }
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{constant::Constant, operator::BinaryOperator},
        rust_ast::{
            block::Block,
            expression::Expression,
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
    fn should_format_let_binding() {
        let let_binding = Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: true,
                identifier: String::from("x"),
            },
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = String::from("let mut x = 1i64;");
        assert_eq!(format!("{}", let_binding), control)
    }

    #[test]
    fn should_format_last_expression_statement() {
        let statement = Statement::ExpressionLast(Expression::Identifier {
            identifier: String::from("x"),
        });
        let control = String::from("x");
        assert_eq!(format!("{}", statement), control)
    }

    #[test]
    fn should_format_intern_expression_statement() {
        let statement = Statement::ExpressionIntern(Expression::Assignement {
            left: Box::new(Expression::Identifier {
                identifier: String::from("x"),
            }),
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Identifier {
                    identifier: String::from("x"),
                }),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Literal {
                    literal: Constant::Integer(1),
                }),
            }),
        });
        let control = String::from("x = x + 1i64;");
        assert_eq!(format!("{}", statement), control)
    }

    #[test]
    fn should_format_enumeration_definition() {
        let item = Statement::Item(Item::Enumeration(Enumeration {
            public_visibility: true,
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Yellow"),
                String::from("Green"),
                String::from("Purple"),
            ],
        }));
        let control = String::from("pub enum Color { Blue, Red, Yellow, Green, Purple }");
        assert_eq!(format!("{}", item), control)
    }

    #[test]
    fn should_format_function_definition() {
        let function = Statement::Item(Item::Function(Function {
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
        }));
        let control = String::from(
            "pub fn foo(x: i64, y: i64) -> i64 { let mut z = x + y; z = z + 1i64; z }",
        );
        assert_eq!(format!("{}", function), control)
    }

    #[test]
    fn should_format_trait_implementation() {
        let r#trait = Statement::Item(Item::Implementation(Implementation {
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
        }));
        let control = String::from("impl Display for Point { type MyString = String; ")
            + "fn fmt(&self, f: &mut String) { write!(f, \"({}, {})\", self.x, self.y); } }";
        assert_eq!(format!("{}", r#trait), control)
    }

    #[test]
    fn should_format_use_import_definition_with_name_path() {
        let use_import = Statement::Item(Item::Import(Import::Use {
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
        }));
        let control = String::from("pub use std::fmt::Debug;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_alias_name_path() {
        let use_import = Statement::Item(Item::Import(Import::Use {
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
        }));
        let control = String::from("pub use std::future::Future as AliasFuture;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_star_path() {
        let use_import = Statement::Item(Item::Import(Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("sync"),
                    tree: Box::new(PathTree::Star),
                }),
            },
        }));
        let control = String::from("pub use std::sync::*;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_group_path() {
        let use_import = Statement::Item(Item::Import(Import::Use {
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
        }));
        let control =
            String::from("pub use std::{ sync::*, fmt::Debug, future::Future as AliasFuture };");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_module_import_definition() {
        let mod_import = Statement::Item(Item::Import(Import::Module {
            public_visibility: true,
            name: String::from("my_module"),
        }));
        let control = String::from("pub mod my_module;");
        assert_eq!(format!("{}", mod_import), control)
    }

    #[test]
    fn should_format_structure_definition() {
        let structure = Statement::Item(Item::Structure(Structure {
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
        }));
        let control = String::from("pub struct Point { pub x: i64, pub y: i64, z: i64 }");
        assert_eq!(format!("{}", structure), control)
    }

    #[test]
    fn should_format_trait_definition() {
        let r#trait = Statement::Item(Item::Trait(Trait {
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
        }));
        let control =
            String::from("pub trait Display { type MyString; fn fmt(&self, f: &mut String); }");
        assert_eq!(format!("{}", r#trait), control)
    }

    #[test]
    fn should_format_type_alias_definition() {
        let alias = Statement::Item(Item::TypeAlias(TypeAlias {
            public_visibility: true,
            name: String::from("Integer"),
            r#type: Type::Identifier {
                identifier: String::from("i64"),
            },
        }));
        let control = String::from("pub type Integer = i64;");
        assert_eq!(format!("{}", alias), control)
    }
}
