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
            Item::Function(_) => todo!(),
            Item::Implementation(_) => todo!(),
            Item::Import(import) => write!(f, "{import}"),
            Item::Structure(structure) => write!(f, "{structure}"),
            Item::Trait(_) => todo!(),
            Item::TypeAlias(type_alias) => write!(f, "{type_alias}"),
        }
    }
}
