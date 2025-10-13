prelude! {}

/// Performs id usage analysis.
pub trait Unused {
    /// Checks that all identifiers are used.
    fn no_unused(&self, ctx: &mut Ctx) -> URes;
}

impl Unused for File {
    fn no_unused(&self, ctx: &mut Ctx) -> URes {
        self.interface.no_unused(ctx)?;
        self.components
            .iter()
            .try_for_each(|component| component.no_unused(ctx))
    }
}

impl Unused for Interface {
    fn no_unused(&self, ctx: &mut Ctx) -> URes {
        self.imports.iter().try_for_each(|(import, flow)| {
            if self
                .services
                .iter()
                .all(|service| service.graph.edges(*import).next().is_none())
            {
                let ident = ctx.get_name(flow.id);
                let e = error!(@ident.loc() => ErrorKind::unused_import(ident.to_string()));
                Err(e)
            } else {
                Ok(())
            }
        })
    }
}

impl Unused for Component {
    fn no_unused(&self, ctx: &mut Ctx) -> URes {
        ctx.get_comp_inputs(self.get_id())
            .iter().chain(ctx.get_comp_locals(self.get_id()))
            .filter(|id| !ctx.is_codegen(**id)).try_for_each(|id| {
                if let Some(graph) = self.get_graph() {
                    if graph.edges_directed(*id, graph::Direction::Incoming).next().is_none(){
                        let ident = ctx.get_name(*id);
                        let comp = ctx.get_name(self.get_id());
                        let e = error!(@ident.loc() => ErrorKind::unused_ident(ident.to_string(), comp.to_string()));
                        return Err(e)
                    }
                }
                Ok(())
            })
    }
}
