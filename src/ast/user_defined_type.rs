use crate::ast::{location::Location, type_system::Type};

#[derive(Debug, PartialEq)]
/// LanGRust user defined type AST.
pub enum UserDefinedType {
    /// Represents a structure definition.
    Structure {
        /// The structure's identifier.
        id: String,
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<(String, Type)>,
        /// Structure location.
        location: Location,
    },
    /// Represents an enumeration definition.
    Enumeration {
        /// The enumeration's identifier.
        id: String,
        /// The enumeration's elements.
        elements: Vec<String>,
        /// Enumeration location.
        location: Location,
    },
    /// Represents an array definition.
    Array {
        /// The array's identifier.
        id: String,
        /// The array's type.
        array_type: Type,
        /// The array's size.
        size: usize,
        /// Array location.
        location: Location,
    },
}

impl UserDefinedType {
    /// Create a [Type] from a [UserDefinedType].
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{location::Location, type_system::Type, user_defined_type::UserDefinedType};
    /// let user_defined_type = UserDefinedType::Structure {
    ///     id: String::from("Point"),
    ///     fields: vec![
    ///         (String::from("x"), Type::Integer),
    ///         (String::from("y"), Type::Integer),
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let new_type = user_defined_type.into_type();
    /// assert_eq!(new_type, Type::Structure(String::from("Point")))
    /// ```
    pub fn into_type(&self) -> Type {
        match self {
            UserDefinedType::Structure {
                id,
                fields: _,
                location: _,
            } => Type::Structure(id.clone()),
            UserDefinedType::Enumeration {
                id,
                elements: _,
                location: _,
            } => Type::Enumeration(id.clone()),
            UserDefinedType::Array {
                id: _,
                array_type,
                size,
                location: _,
            } => Type::Array(Box::new(array_type.clone()), size.clone()),
        }
    }
}

#[cfg(test)]
mod into_type {
    use crate::ast::{location::Location, type_system::Type, user_defined_type::UserDefinedType};

    #[test]
    fn should_construct_structure_type_for_user_defined_structure() {
        let user_defined_type = UserDefinedType::Structure {
            id: String::from("Point"),
            fields: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            location: Location::default(),
        };

        let new_type = user_defined_type.into_type();
        let control = Type::Structure(String::from("Point"));

        assert_eq!(new_type, control);
    }

    #[test]
    fn should_construct_enumeration_type_for_user_defined_enumeration() {
        let user_defined_type = UserDefinedType::Enumeration {
            id: String::from("Color"),
            elements: vec![
                String::from("Red"),
                String::from("Blue"),
                String::from("Green"),
                String::from("Yellow"),
            ],
            location: Location::default(),
        };

        let new_type = user_defined_type.into_type();
        let control = Type::Enumeration(String::from("Color"));

        assert_eq!(new_type, control);
    }

    #[test]
    fn should_construct_array_type_for_user_defined_array() {
        let user_defined_type = UserDefinedType::Array {
            id: String::from("Matrix_3_3"),
            array_type: Type::Array(Box::new(Type::Integer), 3),
            size: 3,
            location: Location::default(),
        };

        let new_type = user_defined_type.into_type();
        let control = Type::Array(Box::new(Type::Array(Box::new(Type::Integer), 3)), 3);

        assert_eq!(new_type, control);
    }
}
