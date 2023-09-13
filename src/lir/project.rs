use super::file::File;

/// A Rust source code project.
pub struct Project {
    /// All the files contained by the project.
    pub files: Vec<File>,
}
