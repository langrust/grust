/// Signals scopes in LanGRust nodes or components.
///
/// A [Scope] is the visibility of the signal in a node/component.
/// It can be:
/// - a [Scope::Input], when it is an input of the node/component
/// - a [Scope::Output] meaning that the signal can be retreived by a
/// node/component application
/// - a [Scope::Local], when it is only reachable in the node/component
/// defining it
/// - but it can also be a [Scope::Memory] signal, only used during
/// compilation to tag a `fby` right expression memory.
///
/// # Example
///
/// ```langrust
/// node blinking(blink_tick_number: int) {
///     change_state: bool = blink_tick_number == prev_tick_state;
///     out on_off_status: int = if status then tick_state else 0;
///
///     prev_tick_state: int = 0 fby tick_state;
///     tick_state: int = if change_state then 1 else prev_tick_state + 1;
///
///     prev_status: bool = false fby status;
///     status: bool = if change_state then !prev_status else prev_status;
/// }
/// ```
/// In the example above, `blink_tick_number` is a [Scope::Input],
/// `on_off_status` is a [Scope::Output] and the other signals are
/// [Scope::Local].
///
/// During the compilation, the compiler will construct intermediate signals.
/// Especially memory signals to replace `fby` expressions:
/// ```langrust
/// prev_tick_state: int = 0 fby tick_state;
/// ```
/// will become
/// ```langrust
/// mem prev_tick_state: int = 0 fby tick_state;
/// ```
/// because it represents the initialized memory of the signal `tick_state`.
/// ```langrust
/// some_signal: int = 0 fby x * y;
/// ```
/// will become
/// ```langrust
/// x_0: int = x * y;
/// mem some_signal: int = 0 fby x_0;
/// ```
/// as it represents the initialized memory of the normalized signal `x_0`.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Scope {
    /// input of the node/component
    Input,
    /// meaning that the signal can be retreived by a node/component application
    Output,
    /// signals that are only reachable in the node/component defining them
    Local,
    /// only used during compilation to indicate that the value is in the buffer
    Memory,
}
