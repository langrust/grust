pub use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, execution_machine::ExecutionMachine,
    function::Function, state_machine::StateMachine, structure::Structure,
};

/// LIR [ArrayAlias](crate::lir::item::array_alias::ArrayAlias) module.
pub mod array_alias;
/// LIR [Enumeration](crate::lir::item::enumeration::Enumeration) module.
pub mod enumeration;
pub mod execution_machine;
/// LIR [Function](crate::lir::item::function::Function) module.
pub mod function;
/// LIR [StateMachine](crate::lir::item::state_machine::StateMachine) module.
pub mod state_machine;
/// LIR [Structure](crate::lir::item::structure::Structure) module.
pub mod structure;

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
