/// Identifiers scopes in GRust components.
///
/// A [Scope] is the visibility of the identifier in a component. It can be:
///
/// - a [Scope::Input], when it is an input of the component
/// - a [Scope::Output] meaning that the identifier can be retreived by a component application
/// - a [Scope::Local], when it is only reachable in the component defining it
/// - but it can also be a [Scope::VeryLocal] identifier, only used during compilation to tag
///   identifiers defined by patterns in `when` or `match`.
#[derive(Debug, PartialEq, Clone)]
pub enum Scope {
    /// Input of the component.
    Input,
    /// Means that the identifier can be retrieved by a component application.
    Output,
    /// Identifiers that are only reachable in the component defining them.
    Local,
    /// Only used during compilation to indicate that the value is not memorizable.
    VeryLocal,
}

impl Scope {
    mk_new! {
        Input: input()
        Output: output()
        Local: local()
        VeryLocal: very_local()
    }
}
