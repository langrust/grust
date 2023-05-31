use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::ast::{
    operator::{BinaryOperator, OtherOperator, UnaryOperator},
    type_system::Type,
};

/// Generate the global context.
///
/// The global context is the [HashMap] storing all functions types, user
/// defined  or builtin functions.
pub fn generate_global_context() -> HashMap<String, Type> {
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
        |t| {
            assert!(elements_context_global
                .insert(t.to_string(), t.get_type())
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
        |t| {
            assert!(elements_context_global
                .insert(t.to_string(), t.get_type())
                .is_none())
        },
    )
}

/// Add other operators to the global context.
///
/// Those operators are builtin functions and must be stored in the global context.
fn add_other_operators_to_global_context(elements_context_global: &mut HashMap<String, Type>) {
    OtherOperator::iter().for_each(|t| {
        assert!(
            // for each operator, try to insert its type in the context
            // and check with `is_none()` that this operator is uniquely
            // defined in the global context
            elements_context_global
                .insert(t.to_string(), t.get_type())
                .is_none()
        )
    })
}
