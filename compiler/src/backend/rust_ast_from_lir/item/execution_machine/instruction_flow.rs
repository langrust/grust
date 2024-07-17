prelude! {
    macro2::Span,
    quote::TokenStreamExt,
    syn::*,
}

prelude! { just
    backend::{
        rust_ast_from_lir::expression::constant_to_syn,
        rust_ast_from_lir::{
            item::execution_machine::flow_expression::rust_ast_from_lir
                as flow_expression_rust_ast_from_lir,
            pattern::rust_ast_from_lir
                as pattern_rust_ast_from_lir,
        },
    },
    lir::item::execution_machine::service_handler::{FlowInstruction, MatchArm},
}

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
            parse_quote! { self.context.#ident = #expression; }
        }
        FlowInstruction::Send(ident, flow_expression) => {
            let enum_ident = Ident::new(to_camel_case(ident.as_str()).as_str(), Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            parse_quote! { self.send_output(O::#enum_ident(#expression, instant)).await?; }
        }
        FlowInstruction::IfThrottle(receiver_name, source_name, delta, instruction) => {
            let receiver_ident = Ident::new(&receiver_name, Span::call_site());
            let source_ident = Ident::new(&source_name, Span::call_site());
            let delta = constant_to_syn(delta);
            let instruction = rust_ast_from_lir(*instruction);

            parse_quote! {
                if (self.context.#receiver_ident - #source_ident).abs() >= #delta {
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
            let mut onchange_tokens = macro2::TokenStream::new();
            onchange_tokens.append_all(onchange_instructions.into_iter().map(rust_ast_from_lir));
            let mut not_onchange_tokens = macro2::TokenStream::new();
            not_onchange_tokens
                .append_all(not_onchange_instructions.into_iter().map(rust_ast_from_lir));
            parse_quote! {
                if self.context.#old_event_ident != #source_ident {
                    #onchange_tokens
                } else {
                    #not_onchange_tokens
                }
            }
        }
        FlowInstruction::ResetTimer(timer_name, ..) => {
            let enum_ident = Ident::new(
                to_camel_case(timer_name.as_str()).as_str(),
                Span::call_site(),
            );
            parse_quote! { self.send_timer(T::#enum_ident, instant).await?; }
        }
        FlowInstruction::ComponentCall(pattern, component_name, events) => {
            let outputs = pattern_rust_ast_from_lir(pattern);
            let component_ident = Ident::new(&component_name, Span::call_site());
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            let args = events.into_iter().map(|opt_event| -> syn::Expr {
                if let Some(event_name) = opt_event {
                    let event_ident = Ident::new(&event_name, Span::call_site());
                    parse_quote! { Some(#event_ident) }
                } else {
                    parse_quote! { None }
                }
            });
            parse_quote! {
                let #outputs = self.#component_ident.step(self.context.#input_getter(#(#args),*));
            }
        }
        FlowInstruction::HandleDelay(input_flows, match_arms) => {
            let input_flows = input_flows.iter().map(|name| -> Expr {
                let ident = Ident::new(name, Span::call_site());
                parse_quote! { self.input_store.#ident.take() }
            });
            let arms = match_arms.into_iter().map(match_arm_to_syn);
            parse_quote! {
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant).await?;
                    match (#(#input_flows),*) {
                        #(#arms)*
                    }
                } else {
                    self.delayed = true;
                }
            }
        }
    }
}

fn match_arm_to_syn(match_arm: MatchArm) -> syn::Arm {
    let MatchArm { patterns, block } = match_arm;
    let syn_pats = patterns.into_iter().map(pattern_rust_ast_from_lir);
    let syn_block = block.into_iter().map(rust_ast_from_lir);
    parse_quote! {
        (#(#syn_pats),*) => {
            #(#syn_block)*
        }
    }
}
