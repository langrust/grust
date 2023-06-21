/// [Color] enumeration used to identify the processing status of an element.
#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    /// Computation has ended.
    Black,
    /// Currently being processed.
    Grey,
    /// Element not processed.
    White,
}
