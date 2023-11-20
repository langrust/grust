/// Node related behaviors.
pub trait Node<I, O>
where
    Self: Sized,
{
    /// Initialize a node state.
    fn init() -> Self;

    /// Perform the step of a state with its input.
    fn step(self, input: I) -> (Self, O);
}
