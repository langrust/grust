use crate::ast::location::Location;
use crate::error::Error;

use std::collections::HashMap;

/// HashMap API handling errors.
///
/// [Context] trait is an API handling errors related to HashMap:
/// - [Error::UnknownElement](crate::error::Error::UnknownElement)
pub trait Context {
    /// The type of the elements in the context.
    type Item;

    /// Returns a reference to the item corresponding to the name or raises an error.
    ///
    /// Raises an [Error::UnknownElement] when the context does not contains an item
    /// corresponding to the name. Otherwise, returns a reference to the item.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::location::Location;
    /// use grustine::common::context::Context;
    ///
    /// let mut context = HashMap::new();
    /// let mut errors = vec![];
    /// let location = Location::default();
    ///
    /// let name = String::from("x");
    /// context.insert(name.clone(), 1);
    ///
    /// context.get_element_or_error(name, location.clone(), &mut errors).unwrap();
    /// context.get_element_or_error(String::from("y"), location, &mut errors).unwrap_err();
    /// ```
    fn get_element_or_error(
        &self,
        name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error>;
}

impl<V> Context for HashMap<String, V> {
    type Item = V;

    fn get_element_or_error(
        &self,
        name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error> {
        match self.get(&name) {
            Some(item) => Ok(item),
            None => {
                let error = Error::UnknownElement {
                    name: name.clone(),
                    location: location.clone(),
                };
                errors.push(error.clone());
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod get_element_or_error {
    use crate::ast::{location::Location, type_system::Type};
    use crate::common::context::Context;
    use std::collections::HashMap;

    #[test]
    fn should_get_reference_when_name_in_context() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

        let name = String::from("x");
        elements_context.insert(name.clone(), Type::Integer);

        let element_type = elements_context
            .get_element_or_error(name, Location::default(), &mut errors)
            .unwrap();

        let control = Type::Integer;

        assert_eq!(*element_type, control);
    }

    #[test]
    fn should_not_add_error_when_name_in_context() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

        let name = String::from("x");
        elements_context.insert(name.clone(), Type::Integer);

        let _ = elements_context
            .get_element_or_error(name, Location::default(), &mut errors)
            .unwrap();

        let control = vec![];

        assert_eq!(errors, control);
    }

    #[test]
    fn should_raise_and_add_error_when_name_not_in_context() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

        let name = String::from("x");
        elements_context.insert(name, Type::Integer);

        let error = elements_context
            .get_element_or_error(String::from("y"), Location::default(), &mut errors)
            .unwrap_err();

        let control = vec![error];

        assert_eq!(errors, control);
    }
}
