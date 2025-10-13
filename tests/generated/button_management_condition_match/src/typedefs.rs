#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    Released,
    Pressed,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResetState {
    InProgress,
    Confirmed,
}
