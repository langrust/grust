use crate::backend::rust_ast_from_lir::statement::rust_ast_from_lir as statement_rust_ast_from_lir;
use crate::lir::item::execution_machine::service_loop::{FlowInstruction, Pattern};
use proc_macro2::Span;
use std::collections::BTreeSet;
use syn::*;

/// Transform LIR instruction on flows into statement.
pub fn rust_ast_from_lir(
    instruction_flow: FlowInstruction,
    crates: &mut BTreeSet<String>,
) -> Vec<syn::Stmt> {
    match instruction_flow {
        FlowInstruction::Let(Pattern::InContext(ident), ..) => {
            let ident = Ident::new(&ident, Span::call_site());
            vec![
                parse_quote!(let mut lock = context.#ident.write().unwrap();),
                parse_quote!(*lock = #ident;),
            ]
        }
        FlowInstruction::Send(event_ident, ..) => {
            let ident: Ident = Ident::new(event_ident.as_str(), Span::call_site());
            let channel: Ident = Ident::new((event_ident + "_channel").as_str(), Span::call_site());
            vec![parse_quote!(#channel.send(#ident).await.unwrap();)]
        }
        FlowInstruction::Let(Pattern::Identifier(_), _) => todo!(),
        FlowInstruction::IfThortle(_, _, _, _) => todo!(),
        FlowInstruction::IfChange(_, _, _) => todo!(),
        FlowInstruction::ResetTimer(_, _) => todo!(),
        FlowInstruction::ComponentCall(_) => todo!(),
    }
}
