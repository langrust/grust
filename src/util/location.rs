use std::ops::Range;

use super::files::FileId;

/// Element location in source code for errors display.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    /// the file identifiant as a [FileId] in case there are multiple files
    pub file_id: FileId,
    /// range in which the element is located
    pub range: Range<usize>,
}
impl Location {
    /// Construct and return the default location.
    pub fn default() -> Self {
        return Location{
            file_id: FileId::default(),
            range: 0..0,
        };
    }
}
