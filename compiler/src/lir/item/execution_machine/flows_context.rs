use std::collections::HashMap;

use crate::common::r#type::Type;

/// A signals context from where components will get their inputs.
#[derive(Debug, PartialEq, Default)]
pub struct FlowsContext {
    pub elements: HashMap<String, Type>,
    pub components: HashMap<String, Vec<(String, String)>>,
}
impl FlowsContext {
    pub fn add_element(&mut self, element_name: String, element_type: &Type) {
        match self.elements.insert(element_name, element_type.clone()) {
            Some(other_ty) => debug_assert!(other_ty.eq(element_type)),
            None => (),
        }
    }
    pub fn add_component(&mut self, component_name: String, input_fields: Vec<(String, String)>) {
        let already_inserted = self.components.insert(component_name, input_fields);
        debug_assert!(already_inserted.is_none())
    }
}
