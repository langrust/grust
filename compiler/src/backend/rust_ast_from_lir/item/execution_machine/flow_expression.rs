use proc_macro2::Span;
use syn::{parse_quote, Expr, Ident};

use crate::{
    backend::rust_ast_from_lir::expression::constant_to_syn,
    lir::item::execution_machine::service_loop::Expression,
};

pub fn rust_ast_from_lir(expression: Expression) -> Expr {
    match expression {
        Expression::Literal { literal } => constant_to_syn(literal),
        Expression::Identifier { identifier } => {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote! { #identifier }
        }
        Expression::InContext { flow } => {
            let flow = Ident::new(&flow, Span::call_site());
            parse_quote! { context.#flow.clone() }
        }
        Expression::TakeFromContext { flow } => {
            let flow = Ident::new(&flow, Span::call_site());
            parse_quote! { std::mem::take(&mut context.#flow.clone()) }
        }
        Expression::Some { expression } => {
            let expression = rust_ast_from_lir(*expression);
            parse_quote! { Some(#expression) }
        }
        Expression::Ok { expression } => {
            let expression = rust_ast_from_lir(*expression);
            parse_quote! { Ok(#expression) }
        }
        Expression::None => parse_quote! { None },
        Expression::Err => parse_quote! { Err(()) },
    }
}
