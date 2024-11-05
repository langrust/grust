use compiler_lir::common::quote::format_ident;

prelude! { just
    macro2::Span, syn, parse_quote, Ident,
    backend::rust_ast_from_lir::expression::constant_to_syn,
    lir::item::execution_machine::service_handler::Expression,
}

pub fn rust_ast_from_lir(expression: Expression) -> syn::Expr {
    match expression {
        Expression::Literal { literal } => constant_to_syn(literal),
        Expression::Event { identifier } => {
            let identifier = format_ident!("{}_ref", identifier);
            parse_quote! { *#identifier }
        }
        Expression::Identifier { identifier } => {
            let identifier = Ident::new(&identifier, Span::call_site());
            parse_quote! { #identifier }
        }
        Expression::InContext { flow } => {
            let flow = Ident::new(&flow, Span::call_site());
            parse_quote! { self.context.#flow.get() }
        }
        Expression::TakeFromContext { flow } => {
            let flow = Ident::new(&flow, Span::call_site());
            parse_quote! { std::mem::take(&mut self.context.#flow.0) }
        }
        Expression::Some { expression } => {
            let expression = rust_ast_from_lir(*expression);
            parse_quote! { Some(#expression) }
        }
        Expression::None => parse_quote! { None },
    }
}
