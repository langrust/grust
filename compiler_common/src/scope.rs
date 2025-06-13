/// Signals scopes in GRust nodes or components.
///
/// A [Scope] is the visibility of the signal in a node/component. It can be:
///
/// - a [Scope::Input], when it is an input of the node/component
/// - a [Scope::Output] meaning that the signal can be retreived by a node/component application
/// - a [Scope::Local], when it is only reachable in the node/component defining it
/// - but it can also be a [Scope::VeryLocal] signal, only used during compilation to tag
///   identifiers defined by patterns in `when` or `match`.
#[derive(Debug, PartialEq, Clone)]
pub enum Scope {
    /// Input of the node/component.
    Input,
    /// Means that the signal can be retrieved by a node/component application.
    Output,
    /// Signals that are only reachable in the node/component defining them.
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
