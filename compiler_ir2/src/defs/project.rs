//! [Project] module.

prelude! {}

/// A project structure.
pub struct Project {
    /// The project's items.
    pub items: Vec<Item>,
}

pub struct ProjectTokens<'a> {
    project: &'a Project,
    ctx: &'a Ctx,
}
impl Project {
    pub fn prepare_tokens<'a>(&'a self, ctx: &'a Ctx) -> ProjectTokens<'a> {
        ProjectTokens { project: self, ctx }
    }
}

impl ToTokens for ProjectTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ctx = self.ctx;
        let mut logic_fun: Option<Vec<_>> = if ctx.conf.mode.greusot() {
            Some(vec![])
        } else {
            None
        };
        macro_rules! add_logic {
            { $($stuff:tt)* } => {
                if let Some(vec) = logic_fun.as_mut() {
                    vec.push($($stuff)*)
                }
            };
        }

        if ctx.conf.mode.greusot() {
            quote!(
                use creusot_contracts::{DeepModel, ensures, logic, open, prelude, requires};
            )
            .to_tokens(tokens)
        }

        for item in self.project.items.iter() {
            match item {
                Item::ExecutionMachine(em) => {
                    if ctx.conf.mode.test() || ctx.conf.mode.demo() {
                        em.prepare_tokens(
                            ctx.conf.mode.demo(),
                            ctx.conf.mode.test(),
                            &ctx.conf.spawn_with,
                            &ctx.conf.get_handle,
                        )
                        .to_tokens(tokens)
                    }
                }
                Item::StateMachine(sm) => sm
                    .prepare_tokens(
                        ctx.conf.mode.greusot(),
                        ctx.conf.align,
                        ctx.conf.public,
                        ctx.conf.tracing,
                    )
                    .to_tokens(tokens),
                Item::Function(fun) => {
                    let (def, logic_opt) = fun.to_def_and_logic_tokens(ctx);
                    def.to_tokens(tokens);
                    if let Some(logic) = logic_opt {
                        add_logic!(logic)
                    }
                }
                Item::Enumeration(enumeration) => enumeration
                    .prepare_tokens(ctx.conf.public, ctx.conf.mode.greusot())
                    .to_tokens(tokens),
                Item::Structure(structure) => structure
                    .prepare_tokens(ctx.conf.public, ctx.conf.mode.greusot())
                    .to_tokens(tokens),
                Item::ArrayAlias(alias) => alias.prepare_tokens(ctx.conf.public).to_tokens(tokens),
            }
        }

        if let Some(logic) = logic_fun {
            quote! {
                mod logical {
                    use creusot_contracts::{open, logic, Int};
                    use super::*;
                    #(#logic)*
                }
            }
            .to_tokens(tokens)
        }
    }
}
