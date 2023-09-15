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
