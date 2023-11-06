use super::file::File;

/// A Rust source code project.
pub struct Project {
    /// All the files contained by the project.
    pub files: Vec<File>,
}
impl Project {
    /// Create a new project.
    pub fn new() -> Self {
        Project { files: vec![] }
    }

    /// Add file.
    pub fn add_file(&mut self, file: File) {
        self.files.push(file)
    }

    /// Generate Rust project.
    pub fn generate(&self) {
        for file in &self.files {
            file.generate()
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Project::new()
    }
}
