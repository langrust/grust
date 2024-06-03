use std::ops::Range;

/// Element location in source code for errors display.
#[derive(Debug, Eq, Hash, Clone)]
pub struct Location {
    /// The file identifier as a [usize] in case there are multiple files.
    pub file_id: usize,
    /// The range in which the element is located.
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
impl serde::Serialize for Location {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_struct("Location")
    }
}
