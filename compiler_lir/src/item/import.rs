prelude! {}

/// A node's import.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Import {
    /// Import of another node.
    StateMachine(String),
    /// Import of a function.
    Function(String),
    /// Import of an enumeration.
    Enumeration(String),
    /// Import of a structure.
    Structure(String),
    /// Import of an array alias.
    ArrayAlias(String),
    /// Import of creusot contracts
    Creusot(String),
}

mk_new! { impl Import =>
    StateMachine: state_machine(s: impl Into<String> = s.into())
    Function: function(s: impl Into<String> = s.into())
    Enumeration: enumeration(s: impl Into<String> = s.into())
    Structure: structure(s: impl Into<String> = s.into())
    ArrayAlias: array_alias(s: impl Into<String> = s.into())
    Creusot: creusot(s: impl Into<String> = s.into())
}
