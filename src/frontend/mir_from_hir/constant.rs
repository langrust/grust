use crate::{common::constant::Constant, mir::item::node_file::import::Import};

impl Constant {
    /// Get imports induced by constant.
    pub fn get_imports(&self) -> Vec<Import> {
        match self {
            Constant::Enumeration(name, _) => {
                vec![Import::Enumeration(name.clone())]
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod get_imports {
    use crate::{common::constant::Constant, mir::item::node_file::import::Import};

    #[test]
    fn should_get_enumeration_import_from_constant_enumeration() {
        let constant = Constant::Enumeration(format!("Color"), format!("Blue"));
        let control = vec![Import::Enumeration(format!("Color"))];
        assert_eq!(constant.get_imports(), control)
    }
}
