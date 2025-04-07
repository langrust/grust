//! [Project] module.

prelude! {}

/// A project structure.
pub struct Project {
    /// The project's items.
    pub items: Vec<Item>,
}

impl Project {
    /// Transform [ir2] item into RustAST item.
    pub fn into_syn(self, ctx: &ir0::Ctx, mut stats: StatsMut) -> Vec<syn::Item> {
        let mut crates = BTreeSet::new();
        let mut rust_items = vec![];
        let mut logic_fun = vec![];

        if ctx.conf.greusot {
            rust_items.push(syn::Item::Use(parse_quote!(
                use creusot_contracts::{DeepModel, ensures, logic, open, prelude, requires};
            )))
        }

        self.items
            .into_iter()
            .enumerate()
            .for_each(|(idx, item)| match item {
                Item::ExecutionMachine(execution_machine) => {
                    if ctx.conf.test || ctx.conf.demo {
                        stats.timed_with(format!("item #{}, execution-machine", idx), |stats| {
                            let item = execution_machine.into_syn(stats);
                            rust_items.push(item);
                        })
                    }
                }
                Item::StateMachine(state_machine) => stats.timed(
                    format!("item #{}, state-machine `{}`", idx, state_machine.name),
                    || {
                        let items = state_machine.into_syn(ctx, &mut crates);
                        rust_items.extend(items);
                    },
                ),
                Item::Function(function) => stats.timed(
                    format!("item #{}, function `{}`", idx, function.name),
                    || {
                        let (item_function, opt_logic) = function.into_syn(ctx, &mut crates);
                        rust_items.push(item_function);
                        if let Some(logic) = opt_logic {
                            logic_fun.push(logic);
                        }
                    },
                ),
                Item::Enumeration(enumeration) => stats.timed(
                    format!("item #{}, enumeration `{}`", idx, enumeration.name),
                    || {
                        let rust_ast_enumeration = enumeration.into_syn(ctx);
                        rust_items.push(syn::Item::Enum(rust_ast_enumeration))
                    },
                ),
                Item::Structure(structure) => stats.timed(
                    format!("item #{}, structure `{}`", idx, structure.name),
                    || {
                        let rust_ast_structure = structure.into_syn(ctx);
                        rust_items.push(syn::Item::Struct(rust_ast_structure))
                    },
                ),
                Item::ArrayAlias(array_alias) => stats.timed(
                    format!("item #{}, array alias `{}`", idx, array_alias.name),
                    || {
                        let rust_ast_array_alias = array_alias.into_syn();
                        rust_items.push(syn::Item::Type(rust_ast_array_alias))
                    },
                ),
            });

        if ctx.conf.greusot {
            let logic_mod: syn::ItemMod = parse_quote! {
                mod logical {
                    use creusot_contracts::{open, logic, Int};
                    use super::*;

                    #(#logic_fun)*
                }
            };
            rust_items.push(syn::Item::Mod(logic_mod));
        }

        stats.timed("item dedup", || {
            // remove duplicated imports
            use itertools::Itertools; // for `unique`
            rust_items.into_iter().unique().collect()
        })
    }
}
