use self::{
    enumeration::Enumeration, function::Function, implementation::Implementation, import::Import,
    r#trait::Trait, structure::Structure, type_alias::TypeAlias,
};

/// LIR [Enumeration](crate::lir::item::enumeration::Enumeration) module.
pub mod enumeration;
/// LIR [Function](crate::lir::item::function::Function) module.
pub mod function;
/// LIR [Implementation](crate::lir::item::implementation::Implementation) module.
pub mod implementation;
/// LIR [Import](crate::lir::item::import::Import) module.
pub mod import;
/// LIR [Signature](crate::lir::item::signature::Signature) module.
pub mod signature;
/// LIR [Structure](crate::lir::item::structure::Structure) module.
pub mod structure;
/// LIR [Trait](crate::lir::item::r#trait::Trait) module.
pub mod r#trait;
/// LIR [TypeAlias](crate::lir::item::type_alias::TypeAlias) module.
pub mod type_alias;

/// All items that can be defined in a module or a scope.
pub enum Item {
    /// An enumeration definition: `enum Color { Blue, Yellow }`.
    Enumeration(Enumeration),
    /// An function definition: `pub fn compute(n: i64) { ... }`.
    Function(Function),
    /// An implementation definition: `impl Clone for Point { ... }`.
    Implementation(Implementation),
    /// An import definition: `use std::sync::Mutex;`.
    Import(Import),
    /// A structure definition: `struct Point { x: i64, y: i64 }`.
    Structure(Structure),
    /// An trait definition: `trait Clone { ... }`.
    Trait(Trait),
    /// A type alias definition: `type MyPoint = Point;`.
    TypeAlias(TypeAlias),
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Enumeration(enumeration) => write!(f, "{enumeration}"),
            Item::Function(function) => write!(f, "{function}"),
            Item::Implementation(implementation) => write!(f, "{implementation}"),
            Item::Import(import) => write!(f, "{import}"),
            Item::Structure(structure) => write!(f, "{structure}"),
            Item::Trait(r#trait) => write!(f, "{}", r#trait),
            Item::TypeAlias(type_alias) => write!(f, "{type_alias}"),
        }
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{constant::Constant, operator::BinaryOperator, r#type::Type as DSLType},
        lir::{
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
    fn should_format_enumeration_definition() {
        let item = Item::Enumeration(Enumeration {
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
        let control = String::from("pub enum Color { Blue, Red, Yellow, Green, Purple }");
        assert_eq!(format!("{}", item), control)
    }

    #[test]
    fn should_format_function_definition() {
        let function = Item::Function(Function {
            signature: Signature {
                public_visibility: true,
                name: String::from("foo"),
                receiver: None,
                inputs: vec![
                    (String::from("x"), Type::Owned(DSLType::Integer)),
                    (String::from("y"), Type::Owned(DSLType::Integer)),
                ],
                output: Type::Owned(DSLType::Integer),
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
        let control = String::from(
            "pub fn foo(x: i64, y: i64) -> i64 { let mut z = x + y; z = z + 1i64; z }",
        );
        assert_eq!(format!("{}", function), control)
    }

    #[test]
    fn should_format_trait_implementation() {
        let r#trait = Item::Implementation(Implementation {
            trait_name: Some(String::from("Display")),
            type_name: String::from("Point"),
            items: vec![
                AssociatedItem::AssociatedType {
                    name: String::from("MyString"),
                    r#type: Type::Owned(DSLType::String),
                },
                AssociatedItem::AssociatedMethod {
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
        let control = String::from("impl Display for Point { type MyString = String; ")
            + "fn fmt(&self, f: &mut String) { write!(f, \"({}, {})\", self.x, self.y); } }";
        assert_eq!(format!("{}", r#trait), control)
    }

    #[test]
    fn should_format_use_import_definition_with_name_path() {
        let use_import = Item::Import(Import::Use {
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
        let control = String::from("pub use std::fmt::Debug;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_alias_name_path() {
        let use_import = Item::Import(Import::Use {
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
        let control = String::from("pub use std::future::Future as AliasFuture;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_star_path() {
        let use_import = Item::Import(Import::Use {
            public_visibility: true,
            tree: PathTree::Path {
                module_name: String::from("std"),
                tree: Box::new(PathTree::Path {
                    module_name: String::from("sync"),
                    tree: Box::new(PathTree::Star),
                }),
            },
        });
        let control = String::from("pub use std::sync::*;");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_use_import_definition_with_group_path() {
        let use_import = Item::Import(Import::Use {
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
        let control =
            String::from("pub use std::{ sync::*, fmt::Debug, future::Future as AliasFuture };");
        assert_eq!(format!("{}", use_import), control)
    }

    #[test]
    fn should_format_module_import_definition() {
        let mod_import = Item::Import(Import::Module {
            public_visibility: true,
            name: String::from("my_module"),
        });
        let control = String::from("pub mod my_module;");
        assert_eq!(format!("{}", mod_import), control)
    }

    #[test]
    fn should_format_structure_definition() {
        let structure = Item::Structure(Structure {
            public_visibility: true,
            name: String::from("Point"),
            fields: vec![
                Field {
                    public_visibility: true,
                    name: String::from("x"),
                    r#type: Type::Owned(DSLType::Integer),
                },
                Field {
                    public_visibility: true,
                    name: String::from("y"),
                    r#type: Type::Owned(DSLType::Integer),
                },
                Field {
                    public_visibility: false,
                    name: String::from("z"),
                    r#type: Type::Owned(DSLType::Integer),
                },
            ],
        });
        let control = String::from("pub struct Point { pub x: i64, pub y: i64, z: i64 }");
        assert_eq!(format!("{}", structure), control)
    }

    #[test]
    fn should_format_trait_definition() {
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
                        inputs: vec![(String::from("f"), Type::MutableReference(DSLType::String))],
                        output: Type::Owned(DSLType::Unit),
                    },
                    default: None,
                },
            ],
        });
        let control =
            String::from("pub trait Display { type MyString; fn fmt(&self, f: &mut String); }");
        assert_eq!(format!("{}", r#trait), control)
    }

    #[test]
    fn should_format_type_alias_definition() {
        let alias = Item::TypeAlias(TypeAlias {
            public_visibility: true,
            name: String::from("Integer"),
            r#type: Type::Owned(DSLType::Integer),
        });
        let control = String::from("pub type Integer = i64;");
        assert_eq!(format!("{}", alias), control)
    }
}
