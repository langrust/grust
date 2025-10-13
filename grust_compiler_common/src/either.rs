/// Sum of two types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Either<U, V> {
    Left(U),
    Right(V),
}
impl<U, V> Either<U, V> {
    /// Converts reference to [Either] into [Either] of references.
    pub fn as_ref(&self) -> Either<&U, &V> {
        match self {
            Either::Left(u) => Either::Left(u),
            Either::Right(v) => Either::Right(v),
        }
    }
    /// Converts mutable reference to [Either] into [Either] of mutable references.
    pub fn as_mut(&mut self) -> Either<&mut U, &mut V> {
        match self {
            Either::Left(u) => Either::Left(u),
            Either::Right(v) => Either::Right(v),
        }
    }
    /// Converts into an option of left value.
    pub fn left(self) -> Option<U> {
        match self {
            Either::Left(u) => Some(u),
            Either::Right(_) => None,
        }
    }
    /// Converts into an option of right value.
    pub fn right(self) -> Option<V> {
        match self {
            Either::Left(_) => None,
            Either::Right(v) => Some(v),
        }
    }
}
