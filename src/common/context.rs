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

    /// Returns a reference to the item corresponding to the name or raises an error.
    ///
    /// Raises an [Error::UnknownSignal] when the context does not contains an item
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
    /// context.get_signal_or_error(name, location.clone(), &mut errors).unwrap();
    /// context.get_signal_or_error(String::from("y"), location, &mut errors).unwrap_err();
    /// ```
    fn get_signal_or_error(
        &self,
        name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error>;

    /// Returns a reference to the item corresponding to the name or raises an error.
    ///
    /// Raises an [Error::UnknownType] when the context does not contains an item
    /// corresponding to the name. Otherwise, returns a reference to the item.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{location::Location, type_system::Type};
    /// use grustine::common::context::Context;
    ///
    /// let mut context = HashMap::new();
    /// let mut errors = vec![];
    /// let location = Location::default();
    ///
    /// let enumeration_name = String::from("Color");
    /// context.insert(enumeration_name.clone(), Type::Enumeration(String::from("Color")));
    ///
    /// context.get_user_type_or_error(enumeration_name, location.clone(), &mut errors).unwrap();
    /// context.get_user_type_or_error(String::from("Day"), location, &mut errors).unwrap_err();
    /// ```
    fn get_user_type_or_error(
        &self,
        name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error>;

    /// Returns a reference to the item corresponding to the name or raises an error.
    ///
    /// Raises an [Error::UnknownType] when the context does not contains an item
    /// corresponding to the name. Otherwise, returns a reference to the item.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{location::Location, type_system::Type};
    /// use grustine::common::context::Context;
    ///
    /// let mut structure_fields = HashMap::new();
    /// let mut errors = vec![];
    /// let location = Location::default();
    ///
    /// let structure_name = String::from("Time");
    /// let field_name = String::from("minute");
    /// structure_fields.insert(field_name.clone(), Type::Integer);
    ///
    /// structure_fields.get_field_or_error(structure_name.clone(), field_name, location.clone(), &mut errors).unwrap();
    /// structure_fields.get_field_or_error(structure_name.clone(), String::from("hour"), location, &mut errors).unwrap_err();
    /// ```
    fn get_field_or_error(
        &self,
        structure_name: String,
        field_name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error>;

    /// Insert the item corresponding to the name or raises an error.
    ///
    /// Raises an [Error::AlreadyDefinedElement] when the context already contains an item
    /// corresponding to the name. Otherwise, insert the item corresponding to the name.
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
    /// context.insert_unique(String::from("y"), 2, location.clone(), &mut errors).unwrap();
    /// context.insert_unique(name, 2, location, &mut errors).unwrap_err();
    /// ```
    fn insert_unique(
        &mut self,
        name: String,
        item: Self::Item,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error>;
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

    fn get_signal_or_error(
        &self,
        name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error> {
        match self.get(&name) {
            Some(item) => Ok(item),
            None => {
                let error = Error::UnknownSignal {
                    name: name.clone(),
                    location: location.clone(),
                };
                errors.push(error.clone());
                Err(error)
            }
        }
    }

    fn get_user_type_or_error(
        &self,
        name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error> {
        match self.get(&name) {
            Some(item) => Ok(item),
            None => {
                let error = Error::UnknownType {
                    name: name.clone(),
                    location: location.clone(),
                };
                errors.push(error.clone());
                Err(error)
            }
        }
    }

    fn get_field_or_error(
        &self,
        structure_name: String,
        field_name: String,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<&Self::Item, Error> {
        match self.get(&field_name) {
            Some(item) => Ok(item),
            None => {
                let error = Error::UnknownField {
                    structure_name,
                    field_name,
                    location: location.clone(),
                };
                errors.push(error.clone());
                Err(error)
            }
        }
    }

    fn insert_unique(
        &mut self,
        name: String,
        item: Self::Item,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self.get(&name) {
            Some(_) => {
                let error = Error::AlreadyDefinedElement {
                    name: name.clone(),
                    location: location.clone(),
                };
                errors.push(error.clone());
                Err(error)
            }
            None => {
                self.insert(name, item);
                Ok(())
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

#[cfg(test)]
mod get_field_or_error {
    use crate::ast::{location::Location, type_system::Type};
    use crate::common::context::Context;
    use std::collections::HashMap;

    #[test]
    fn should_get_reference_when_field_in_structure() {
        let mut errors = vec![];
        let mut structure_fields = HashMap::new();

        let structure_name = String::from("Point");
        structure_fields.insert(String::from("x"), Type::Integer);
        structure_fields.insert(String::from("y"), Type::Integer);

        let field_type = structure_fields
            .get_field_or_error(
                structure_name,
                String::from("x"),
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Type::Integer;

        assert_eq!(*field_type, control);
    }

    #[test]
    fn should_not_add_error_when_field_in_structure() {
        let mut errors = vec![];
        let mut structure_fields = HashMap::new();

        let structure_name = String::from("Point");
        structure_fields.insert(String::from("x"), Type::Integer);
        structure_fields.insert(String::from("y"), Type::Integer);

        let _ = structure_fields
            .get_field_or_error(
                structure_name,
                String::from("x"),
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = vec![];

        assert_eq!(errors, control);
    }

    #[test]
    fn should_raise_and_add_error_when_field_not_in_structure() {
        let mut errors = vec![];
        let mut structure_fields = HashMap::new();

        let structure_name = String::from("Point");
        structure_fields.insert(String::from("x"), Type::Integer);
        structure_fields.insert(String::from("y"), Type::Integer);

        let error = structure_fields
            .get_field_or_error(
                structure_name,
                String::from("z"),
                Location::default(),
                &mut errors,
            )
            .unwrap_err();

        let control = vec![error];

        assert_eq!(errors, control);
    }
}

#[cfg(test)]
mod insert_unique {
    use crate::ast::{location::Location, type_system::Type};
    use crate::common::context::Context;
    use std::collections::HashMap;

    #[test]
    fn should_insert_item_when_name_not_in_context() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

        let name = String::from("x");
        elements_context.insert(name, Type::Integer);

        elements_context
            .insert_unique(
                String::from("y"),
                Type::Float,
                Location::default(),
                &mut errors,
            )
            .unwrap();
    }

    #[test]
    fn should_not_add_error_when_name_not_in_context() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

        let name = String::from("x");
        elements_context.insert(name, Type::Integer);

        elements_context
            .insert_unique(
                String::from("y"),
                Type::Float,
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = vec![];

        assert_eq!(errors, control);
    }

    #[test]
    fn should_raise_error_when_name_in_context() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

        let name = String::from("x");
        elements_context.insert(name, Type::Integer);

        let error = elements_context
            .insert_unique(
                String::from("x"),
                Type::Float,
                Location::default(),
                &mut errors,
            )
            .unwrap_err();

        let control = vec![error];

        assert_eq!(errors, control);
    }
}
