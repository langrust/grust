use std::collections::HashMap;

use crate::common::r#type::Type;

/// A signals context from where components will get their inputs.
#[derive(Debug, PartialEq, Default)]
pub struct SignalsContext {
    pub elements: HashMap<String, Type>,
}
