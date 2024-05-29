use crate::backend::rust_ast_from_lir::expression::constant_to_syn;
use crate::backend::rust_ast_from_lir::{
    item::execution_machine::flow_expression::rust_ast_from_lir as flow_expression_rust_ast_from_lir,
    pattern::rust_ast_from_lir as pattern_rust_ast_from_lir,
};
use crate::lir::item::execution_machine::service_loop::FlowInstruction;
use proc_macro2::Span;
use quote::TokenStreamExt;
use syn::*;

/// Transform LIR instruction on flows into statement.
pub fn rust_ast_from_lir(instruction_flow: FlowInstruction) -> syn::Stmt {
    match instruction_flow {
        FlowInstruction::Let(ident, flow_expression) => {
            let ident = Ident::new(&ident, Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            parse_quote! { let #ident = #expression; }
        }
        FlowInstruction::UpdateContext(ident, flow_expression) => {
            let ident = Ident::new(&ident, Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            parse_quote! { context.#ident = #expression; }
        }
        FlowInstruction::Send(ident, flow_expression) => {
            let channel: Ident = Ident::new((ident + "_channel").as_str(), Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            parse_quote!(#channel.send(#expression).await.unwrap();)
        }
        FlowInstruction::IfThrotle(receiver_name, source_name, delta, instruction) => {
            let receiver_ident = Ident::new(&receiver_name, Span::call_site());
            let source_ident = Ident::new(&source_name, Span::call_site());
            let delta = constant_to_syn(delta);
            let instruction = rust_ast_from_lir(*instruction);

            parse_quote! {
                if (context.#receiver_ident - #source_ident) >= #delta {
                    #instruction
                }
            }
        }
        FlowInstruction::IfChange(
            old_event_name,
            source_name,
            onchange_instructions,
            not_onchange_instructions,
        ) => {
            let old_event_ident = Ident::new(&old_event_name, Span::call_site());
            let source_ident = Ident::new(&source_name, Span::call_site());
            let mut onchange_tokens = proc_macro2::TokenStream::new();
            onchange_tokens.append_all(onchange_instructions.into_iter().map(rust_ast_from_lir));
            let mut not_onchange_tokens = proc_macro2::TokenStream::new();
            not_onchange_tokens
                .append_all(not_onchange_instructions.into_iter().map(rust_ast_from_lir));
            parse_quote! {
                if context.#old_event_ident != #source_ident {
                    #onchange_tokens
                } else {
                    #not_onchange_tokens
                }
            }
        }
        FlowInstruction::ResetTimer(timer_name, deadline) => {
            let timer_ident = Ident::new(&timer_name, Span::call_site());
            let deadline = Expr::Lit(ExprLit {
                attrs: vec![],
                lit: syn::Lit::Int(LitInt::new(&format!("{deadline}u64"), Span::call_site())),
            });
            parse_quote! {
                #timer_ident.reset(Instant::now() + Duration::from_millis(#deadline));
            }
        }
        FlowInstruction::EventComponentCall(pattern, component_name, optional_event) => {
            let outputs = pattern_rust_ast_from_lir(pattern);
            let component_ident = Ident::new(&component_name, Span::call_site());
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            if let Some(event_name) = optional_event {
                let event_ident = Ident::new(&event_name, Span::call_site());
                parse_quote! {
                    let #outputs = #component_ident.step(context.#input_getter(Some(#event_ident)));
                }
            } else {
                parse_quote! {
                    let #outputs = #component_ident.step(context.#input_getter(None));
                }
            }
        }
        FlowInstruction::ComponentCall(pattern, component_name) => {
            let outputs = pattern_rust_ast_from_lir(pattern);
            let component_ident = Ident::new(&component_name, Span::call_site());
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            parse_quote! {
                let #outputs = #component_ident.step(context.#input_getter());
            }
        }
    }
}
