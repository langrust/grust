//! [Item] module.

prelude! {}

pub use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, function::Function, structure::Structure,
};

mod array_alias;
mod enumeration;
mod function;
mod structure;

/// An item of the project.
#[derive(Debug, PartialEq)]
pub enum Item {
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
