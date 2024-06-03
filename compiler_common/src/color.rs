/// [Color] enumeration used to identify the processing status of an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    /// Computation has ended.
    Black,
    /// Currently being processed.
    Grey,
    /// Element not processed.
    White,
}

impl Color {
    mk_new! {
        Black: black()
        Grey: grey()
        White: white()
    }
}
