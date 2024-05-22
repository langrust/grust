use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, function::Function, state_machine::StateMachine,
    structure::Structure,
};

/// LIR [ArrayAlias](crate::lir::item::array_alias::ArrayAlias) module.
pub mod array_alias;
/// LIR [Enumeration](crate::lir::item::enumeration::Enumeration) module.
pub mod enumeration;
/// LIR [Function](crate::lir::item::function::Function) module.
pub mod function;
/// LIR [Import](crate::lir::item::import::Import) module.
pub mod import;
/// LIR [StateMachine](crate::lir::item::state_machine::StateMachine) module.
pub mod state_machine;
/// LIR [Structure](crate::lir::item::structure::Structure) module.
pub mod structure;

/// An item of the project.
#[derive(Debug, PartialEq)]
pub enum Item {
    /// A node file.
    StateMachine(StateMachine),
    /// A function definition.
    Function(Function),
    /// An enumeration definition.
    Enumeration(Enumeration),
    /// A structure definition.
    Structure(Structure),
    /// An array alias definition.
    ArrayAlias(ArrayAlias),
}
