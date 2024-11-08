//! [Item] module.

prelude! {}

pub use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, function::Function, import::Import,
    structure::Structure,
};

mod array_alias;
mod enumeration;
mod function;
mod import;
mod structure;

/// An item of the project.
#[derive(Debug, PartialEq)]
pub enum Item {
    /// Import a state-machine.
    Import(Import),
    /// A state-machine.
    StateMachine(StateMachine),
    /// Ax execution-machine.
    ExecutionMachine(ExecutionMachine),
    /// A function definition.
    Function(Function),
    /// An enumeration definition.
    Enumeration(Enumeration),
    /// A structure definition.
    Structure(Structure),
    /// An array alias definition.
    ArrayAlias(ArrayAlias),
}
