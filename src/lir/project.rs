use super::file::File;

/// A Rust source code project.
pub struct Project {
    /// All the files contained by the project.
    pub files: Vec<File>,
}
impl Project {
    /// Generate Rust project.
    pub fn generate(&self) {
        for file in &self.files {
            file.generate()
        }
    }
}
