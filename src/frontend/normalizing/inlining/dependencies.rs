use std::collections::HashMap;

use crate::hir::{dependencies::Dependencies, stream_expression::StreamExpression};

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
        context_map: &HashMap<String, Union<Signal, StreamExpression>>,
    ) {
        let new_dependencies = self
            .get()
            .unwrap()
            .iter()
            .flat_map(|(id, depth)| match context_map.get(id) {
                Some(Union::I1(Signal { id: new_id, .. })) => vec![(new_id.clone(), *depth)],
                Some(Union::I2(expression)) => expression
                    .get_dependencies()
                    .iter()
                    .map(|(new_id, new_depth)| (new_id.clone(), depth + new_depth))
                    .collect(),
                None => vec![],
            })
            .collect();

        *self = Dependencies::from(new_dependencies);
    }
}
