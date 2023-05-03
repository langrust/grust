use crate::util::location::Location;

use super::component::Component;

#[derive(Debug, PartialEq)]

/// Enumerates the different kinds of files in LanGRust.
pub enum File {
    /// A LanGRust [File::Module] is composed of todo!()
    Module{
        // todo!()
        /// Module location.
        location: Location,
    },
    /// A LanGRust [File::Program] is composed of todo!()
    Program{
        // todo!()
        /// Program component. It represents the system.
        component: Component,
        /// Program location.
        location: Location,
    },
}
