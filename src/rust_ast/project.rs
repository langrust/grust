use super::file::File;

#[derive(serde::Serialize)]
/// A Rust source code project.
pub struct Project {
    /// Project's directory.
    pub directory: String,
    /// All the files contained by the project.
    pub files: Vec<File>,
}
impl Project {
    /// Create a new project.
    pub fn new() -> Self {
        Project {
            files: vec![],
            directory: String::new(),
        }
    }

    /// Add file.
    pub fn add_file(&mut self, file: File) {
        self.files.push(file)
    }

    /// Set project's directory.
    pub fn set_parent<P>(&mut self, path: P)
    where
        P: AsRef<std::path::Path>,
    {
        let subdirectory = std::mem::take(&mut self.directory);
        self.directory = path
            .as_ref()
            .join(subdirectory)
            .into_os_string()
            .into_string()
            .unwrap();
        self.files
            .iter_mut()
            .for_each(|file| file.set_parent(&path))
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
