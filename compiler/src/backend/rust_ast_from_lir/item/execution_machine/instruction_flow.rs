prelude! {
    macro2::Span, syn::*, quote::format_ident,
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
pub fn rust_ast_from_lir(instruction_flow: FlowInstruction) -> Vec<syn::Stmt> {
    let stmt = match instruction_flow {
        FlowInstruction::Let(ident, flow_expression) => {
            let ident = Ident::new(&ident, Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            parse_quote! { let #ident = #expression; }
        }
        FlowInstruction::InitEvent(ident) => {
            let ident = format_ident!("{}_ref", ident);
            parse_quote! { let #ident = &mut None; }
        }
        FlowInstruction::UpdateEvent(ident, expr) => {
            let ident = format_ident!("{}_ref", ident);
            let expression = flow_expression_rust_ast_from_lir(expr);
            parse_quote! { *#ident = #expression; }
        }
        FlowInstruction::UpdateContext(ident, flow_expression) => {
            let ident = Ident::new(&ident, Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            parse_quote! { self.context.#ident.set(#expression); }
        }
        FlowInstruction::Send(ident, flow_expression, instant) => {
            let enum_ident = Ident::new(to_camel_case(ident.as_str()).as_str(), Span::call_site());
            let expression = flow_expression_rust_ast_from_lir(flow_expression);
            let instant = if let Some(instant) = instant {
                format_ident!("{instant}_instant")
            } else {
                Ident::new("instant", Span::call_site())
            };
            parse_quote! { self.send_output(O::#enum_ident(#expression, #instant)).await?; }
        }
        FlowInstruction::IfThrottle(receiver_name, source_name, delta, instruction) => {
            let receiver_ident = Ident::new(&receiver_name, Span::call_site());
            let source_ident = Ident::new(&source_name, Span::call_site());
            let delta = constant_to_syn(delta);
            let instructions = rust_ast_from_lir(*instruction);

            parse_quote! {
                if (self.context.#receiver_ident.get() - #source_ident).abs() >= #delta {
                    #(#instructions)*
                }
            }
        }
        FlowInstruction::IfChange(old_event_name, signal, then) => {
            let old_event_ident = Ident::new(&old_event_name, Span::call_site());
            let expr = flow_expression_rust_ast_from_lir(signal);
            let then = rust_ast_from_lir(*then);
            parse_quote! {
                if self.context.#old_event_ident.get() != #expr {
                    #(#then)*
                }
            }
        }
        FlowInstruction::ResetTimer(timer_name, import_name) => {
            let enum_ident = Ident::new(
                to_camel_case(timer_name.as_str()).as_str(),
                Span::call_site(),
            );
            let instant = format_ident!("{import_name}_instant");
            parse_quote! { self.send_timer(T::#enum_ident, #instant).await?; }
        }
        FlowInstruction::ComponentCall(pattern, component_name, events) => {
            let outputs = pattern_rust_ast_from_lir(pattern);
            let component_ident = Ident::new(&component_name, Span::call_site());
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            let args = events.into_iter().map(|opt_event| -> syn::Expr {
                if let Some(event_name) = opt_event {
                    let event_ident = format_ident!("{event_name}_ref");
                    parse_quote! { *#event_ident }
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
        FlowInstruction::IfActivated(events, signals, then, els) => {
            let actv_cond = events
                .iter()
                .map(|e| -> Expr {
                    let ident = format_ident!("{e}_ref");
                    parse_quote! { #ident.is_some() }
                })
                .chain(signals.iter().map(|s| -> Expr {
                    let ident = Ident::new(s, Span::call_site());
                    parse_quote! { self.context.#ident.is_new() }
                }));
            let then_instrs = rust_ast_from_lir(*then);

            if events.is_empty() && signals.is_empty() {
                return els.map_or(vec![], |instr| rust_ast_from_lir(*instr));
            } else {
                if let Some(instr) = els {
                    let els_instrs = rust_ast_from_lir(*instr);
                    parse_quote! {
                        if #(#actv_cond)||* {
                            #(#then_instrs)*
                        } else {
                            #(#els_instrs)*
                        }
                    }
                } else {
                    parse_quote! {
                        if #(#actv_cond)||* {
                            #(#then_instrs)*
                        }
                    }
                }
            }
        }
        FlowInstruction::Seq(instrs) => {
            return instrs.into_iter().flat_map(rust_ast_from_lir).collect()
        }
        FlowInstruction::Para(_method_map) => {
            parse_quote! {
                todo!();
            }
        }
    };
    vec![stmt]
}

fn match_arm_to_syn(match_arm: MatchArm) -> syn::Arm {
    let MatchArm { patterns, instr } = match_arm;
    let syn_pats = patterns.into_iter().map(pattern_rust_ast_from_lir);
    let stmts = rust_ast_from_lir(instr);
    parse_quote! {
        (#(#syn_pats),*) => {
            #(#stmts)*
        }
    }
}
