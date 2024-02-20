use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::common::{
    operator::{BinaryOperator, OtherOperator, UnaryOperator},
    r#type::Type,
};

/// Generate the global context.
///
/// The global context is the [HashMap] storing all builtin functions types.
pub fn generate() -> HashMap<String, Type> {
    let mut elements_context_global = HashMap::new();
    add_binary_operators_to_global_context(&mut elements_context_global);
    add_unary_operators_to_global_context(&mut elements_context_global);
    add_other_operators_to_global_context(&mut elements_context_global);
    elements_context_global
}

/// Add binary operators to the global context.
///
/// Binary operators are builtin functions and must be stored in the global context.
fn add_binary_operators_to_global_context(elements_context_global: &mut HashMap<String, Type>) {
    BinaryOperator::iter().for_each(
        // for each unary operator, try to insert its type in the context
        // and check with `is_none()` that this operator is uniquely
        // defined in the global context
        |operator| {
            debug_assert!(elements_context_global
                .insert(operator.to_string(), operator.get_type())
                .is_none())
        },
    )
}

/// Add unary operators to the global context.
///
/// Unary operators are builtin functions and must be stored in the global context.
fn add_unary_operators_to_global_context(elements_context_global: &mut HashMap<String, Type>) {
    UnaryOperator::iter().for_each(
        // for each unary operator, try to insert its type in the context
        // and check with `is_none()` that this operator is uniquely
        // defined in the global context
        |operator| {
            debug_assert!(elements_context_global
                .insert(operator.to_string(), operator.get_type())
                .is_none())
        },
    )
}

/// Add other operators to the global context.
///
/// Those operators are builtin functions and must be stored in the global context.
fn add_other_operators_to_global_context(elements_context_global: &mut HashMap<String, Type>) {
    OtherOperator::iter().for_each(|operator| {
        debug_assert!(
            // for each operator, try to insert its type in the context
            // and check with `is_none()` that this operator is uniquely
            // defined in the global context
            elements_context_global
                .insert(operator.to_string(), operator.get_type())
                .is_none()
        )
    })
}
