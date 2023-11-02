use std::ops::Range;

/// Element location in source code for errors display.
#[derive(Debug, Eq, Clone, serde::Serialize)]
pub struct Location {
    /// the file identifiant as a [usize] in case there are multiple files
    pub file_id: usize,
    /// range in which the element is located
    pub range: Range<usize>,
}
impl Default for Location {
    fn default() -> Self {
        Location {
            file_id: 0,
            range: 0..0,
        }
    }
}
impl PartialEq for Location {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
