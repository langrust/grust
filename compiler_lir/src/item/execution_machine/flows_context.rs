prelude! {}

/// A signals context from where components will get their inputs.
#[derive(Debug, PartialEq, Default)]
pub struct FlowsContext {
    pub elements: HashMap<String, Typ>,
    pub components: HashMap<String, (Vec<(String, String, Typ)>, Vec<(String, String)>)>,
}
impl FlowsContext {
    pub fn add_element(&mut self, element_name: String, element_type: &Typ) {
        match self.elements.insert(element_name, element_type.clone()) {
            Some(other_ty) => debug_assert!(other_ty.eq(element_type)),
            None => (),
        }
    }
    pub fn contains_element(&self, element_name: &String) -> bool {
        self.elements.contains_key(element_name)
    }
    pub fn add_component(
        &mut self,
        component_name: String,
        events_fields: Vec<(String, String, Typ)>,
        signals_fields: Vec<(String, String)>,
    ) {
        let already_inserted = self
            .components
            .insert(component_name, (events_fields, signals_fields));
        debug_assert!(already_inserted.is_none())
    }
}
