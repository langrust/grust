//! HIR [File](crate::hir::file::File) module.

prelude! {
    hir::{function::Function, interface::Interface, component::Component, typedef::Typedef},
}

/// A LanGRust [File] is composed of functions, components,
/// types defined by the user, components and interface.
pub struct File {
    /// Program types.
    pub typedefs: Vec<Typedef>,
    /// Program functions.
    pub functions: Vec<Function>,
    /// Program components. They are functional requirements.
    pub components: Vec<Component>,
    /// Program interface. It represents the system.
    pub interface: Interface,
    /// Program location.
    pub location: Location,
}

impl File {
    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.components
            .iter()
            .filter_map(|component| match component {
                Component::Definition(comp_def) => Some(comp_def),
                Component::Import(_) => None,
            })
            .all(|component| component.no_fby())
    }
    /// Tell if it is in normal form.
    ///
    /// - component application as root expression
    /// - no rising edge
    pub fn is_normal_form(&self) -> bool {
        self.components
            .iter()
            .filter_map(|component| match component {
                Component::Definition(comp_def) => Some(comp_def),
                Component::Import(_) => None,
            })
            .all(|component| component.is_normal_form())
    }
    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        self.components
            .iter()
            .filter_map(|component| match component {
                Component::Definition(comp_def) => Some(comp_def),
                Component::Import(_) => None,
            })
            .all(|component| component.no_component_application())
    }
}
