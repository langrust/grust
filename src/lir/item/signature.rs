use crate::lir::r#type::Type;

/// Function or method signature.
pub struct Signature {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Function or method's name.
    pub name: String,
    /// Optional `self` input.
    pub receiver: Option<Receiver>,
    /// List of inputs.
    pub inputs: Vec<(String, Type)>,
    /// Returned type.
    pub output: Type,
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        let receiver = if let Some(receiver) = &self.receiver {
            format!("{receiver}, ")
        } else {
            "".to_string()
        };
        let inputs = self
            .inputs
            .iter()
            .map(|(id, r#type)| format!("{id}: {}", r#type))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "{}fn {}({}{}) -> {}",
            visibility, self.name, receiver, inputs, self.output
        )
    }
}

/// The `self` argument of a method.
pub struct Receiver {
    /// Reference: `true` is reference, `false` is owned.
    pub reference: bool,
    /// Mutability: `true` is mutable, `false` is immutable.
    pub mutable: bool,
}

impl std::fmt::Display for Receiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reference = if self.reference { "&" } else { "" };
        let mutable = if self.mutable { "mut " } else { "" };
        write!(f, "{}{}self", reference, mutable)
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::r#type::Type as DSLType,
        lir::{
            item::signature::{Receiver, Signature},
            r#type::Type,
        },
    };

    #[test]
    fn should_format_signature_without_receiver() {
        let signature = Signature {
            public_visibility: true,
            name: String::from("foo"),
            receiver: None,
            inputs: vec![
                (String::from("x"), Type::Owned(DSLType::Integer)),
                (String::from("y"), Type::Owned(DSLType::Integer)),
            ],
            output: Type::Owned(DSLType::Integer),
        };
        let control = String::from("pub fn foo(x: i64, y: i64) -> i64");
        assert_eq!(format!("{}", signature), control)
    }

    #[test]
    fn should_format_signature_with_receiver() {
        let signature = Signature {
            public_visibility: true,
            name: String::from("foo"),
            receiver: Some(Receiver {
                reference: true,
                mutable: true,
            }),
            inputs: vec![(String::from("y"), Type::Owned(DSLType::Integer))],
            output: Type::Owned(DSLType::Integer),
        };
        let control = String::from("pub fn foo(&mut self, y: i64) -> i64");
        assert_eq!(format!("{}", signature), control)
    }
}
