/// Component trait to implement.
///
/// It defines the state machine primitives of components:
/// - [Component::init] creates a new initialized state; and
/// - [Component::step] performs a step in the state machine, returning the output and updating the state.
///
/// # Example
///
/// For a component `counter` as follows
///
/// ```grust
/// component counter(res: bool, tick: bool) -> (o: int) {
///     init o = 0;
///     o = if res then 0 else add(last o, inc);
///     let inc: int = if tick then 1 else 0;
/// }
/// ```
///
/// the implementation of the [Component] trait should resemble to:
///
/// ```rust
/// use grust_core::Component;
///
/// pub struct CounterState {
///     last_o: i64,
/// }
/// pub struct CounterInput {
///     pub res: bool,
///     pub tick: bool,
/// }
/// impl Component for CounterState {
///     type Input = CounterInput;
///     type Output = i64;
///
///     fn init() -> CounterState {
///         CounterState { last_o: 0i64 }
///     }
///     fn step(&mut self, input: CounterInput) -> i64 {
///         let inc = if input.tick { 1i64 } else { 0i64 };
///         let o = if input.res {
///             0i64
///         } else {
///             self.last_o + inc
///         };
///         self.last_o = o;
///         o
///     }
/// }
/// ```
pub trait Component {
    type Input;
    type Output;
    fn init() -> Self;
    fn step(&mut self, input: Self::Input) -> Self::Output;
}
