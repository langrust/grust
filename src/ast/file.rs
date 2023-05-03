use crate::util::location::Location;

#[derive(Debug, PartialEq)]

/// Enumerates the different kinds of files in LanGRust.
pub enum File {
    /// A LanGRust [File::Module] is composed of todo!()
    Module(
        // todo!()
        Location
    ),
    /// A LanGRust [File::Program] is composed of todo!()
    Program(
        // todo!()
        Location
    ),
}
