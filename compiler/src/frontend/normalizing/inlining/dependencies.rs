prelude! {
    common::HashMap,
    hir::{dependencies::Dependencies, stream_expression::StreamExpression},
}

use super::Union;

impl Dependencies {
    /// Replace identifier occurence by dependencies of element in context.
    ///
    /// It will modify the dependencies according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the dependencies of the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` which depends
    /// on `x` and `y` will depends on `a` and `b`.
    pub fn replace_by_context(
        &mut self,
        context_map: &HashMap<usize, Union<usize, StreamExpression>>,
    ) {
        let new_dependencies = self
            .get()
            .unwrap()
            .iter()
            .flat_map(|(id, label)| match context_map.get(id) {
                Some(Union::I1(new_id)) => vec![(*new_id, label.clone())],
                Some(Union::I2(expression)) => expression
                    .get_dependencies()
                    .iter()
                    .map(|(new_id, new_label)| (new_id.clone(), label.add(new_label)))
                    .collect(),
                None => vec![],
            })
            .collect();

        *self = Dependencies::from(new_dependencies);
    }
}
