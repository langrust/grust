use std::collections::HashMap;

use crate::ast::pattern::Pattern;
use crate::common::scope::Scope;

impl Pattern {
    /// Fill the context with locally defined signals in the pattern.
    pub fn fill_context(&self, elements_context: &mut HashMap<String, Scope>) {
        match self {
            Pattern::Identifier { name, .. } => assert!(elements_context
                .insert(name.clone(), Scope::Local,)
                .is_none()),
            Pattern::Structure { fields, .. } => fields
                .iter()
                .for_each(|(_, pattern)| pattern.fill_context(elements_context)),
            Pattern::Some { pattern, .. } => pattern.fill_context(elements_context),
            _ => (),
        }
    }
}

#[cfg(test)]
mod fill_context {
    use std::collections::HashMap;

    use crate::ast::pattern::Pattern;
    use crate::common::{constant::Constant, location::Location, scope::Scope};

    #[test]
    fn should_add_identifier_to_context_as_local_signal() {
        let mut elements_context = HashMap::new();

        let given_pattern = Pattern::Identifier {
            name: String::from("y"),
            location: Location::default(),
        };

        given_pattern.fill_context(&mut elements_context);

        let y = String::from("y");
        assert_eq!(elements_context[&y], Scope::Local);
    }

    #[test]
    fn should_add_structure_identifier_to_context_as_local_signal() {
        let mut elements_context = HashMap::new();

        let given_pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Pattern::Constant {
                        constant: Constant::Integer(1),
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Pattern::Identifier {
                        name: String::from("y"),
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        given_pattern.fill_context(&mut elements_context);

        let y = String::from("y");
        assert_eq!(elements_context[&y], Scope::Local);
    }

    #[test]
    fn should_add_option_identifier_to_context_as_local_signal() {
        let mut elements_context = HashMap::new();

        let given_pattern = Pattern::Some {
            pattern: Box::new(Pattern::Identifier {
                name: String::from("y"),
                location: Location::default(),
            }),
            location: Location::default(),
        };

        given_pattern.fill_context(&mut elements_context);

        let y = String::from("y");
        assert_eq!(elements_context[&y], Scope::Local);
    }
}
