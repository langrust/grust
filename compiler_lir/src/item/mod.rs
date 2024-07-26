//! LIR [Item] module.

pub use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, execution_machine::ExecutionMachine,
    function::Function, import::Import, state_machine::StateMachine, structure::Structure,
};

pub mod array_alias;
pub mod enumeration;
pub mod execution_machine;
pub mod function;
pub mod import;
pub mod state_machine;
pub mod structure;

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
