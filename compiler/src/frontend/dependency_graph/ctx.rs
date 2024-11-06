//! Notion of context(s) for expression traversal and analysis.

prelude! {
    graph::*,
}

pub type Graph = DiGraphMap<usize, Label>;
pub type ReducedGraph = HashMap<usize, Graph>;
pub type ProcManager = HashMap<usize, Color>;

pub struct Ctx<'a> {
    pub symbol_table: &'a SymbolTable,
    pub reduced_graphs: &'a mut ReducedGraph,
    pub errors: &'a mut Vec<Error>,
}
mk_new! { impl{'a} Ctx<'a> =>
    new {
        symbol_table: &'a SymbolTable,
        reduced_graphs: &'a mut ReducedGraph,
        errors: &'a mut Vec<Error>,
    }
}
impl<'a> Ctx<'a> {
    pub fn as_graph_ctx<'g>(&'g mut self, graph: &'g mut Graph) -> GraphCtx<'a, 'g> {
        GraphCtx { ctx: self, graph }
    }
}

pub struct GraphCtx<'a, 'graph> {
    pub ctx: &'graph mut Ctx<'a>,
    pub graph: &'graph mut Graph,
}
impl<'a, 'g> GraphCtx<'a, 'g> {
    pub fn new(ctx: &'a mut Ctx<'a>, graph: &'g mut Graph) -> Self {
        Self { ctx, graph }
    }
}
impl<'a, 'g> std::ops::Deref for GraphCtx<'a, 'g> {
    type Target = Ctx<'a>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl<'a, 'g> std::ops::DerefMut for GraphCtx<'a, 'g> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}
impl<'a, 'g> GraphCtx<'a, 'g> {
    pub fn as_proc_ctx<'p>(
        &'p mut self,
        proc_manager: &'p mut ProcManager,
    ) -> GraphProcCtx<'a, 'g, 'p> {
        GraphProcCtx {
            ctx: self,
            proc_manager,
        }
    }
}

pub struct GraphProcCtx<'a, 'graph, 'proc> {
    ctx: &'proc mut GraphCtx<'a, 'graph>,
    pub proc_manager: &'proc mut ProcManager,
}
impl<'a, 'g, 'p> std::ops::Deref for GraphProcCtx<'a, 'g, 'p> {
    type Target = GraphCtx<'a, 'g>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl<'a, 'g, 'p> std::ops::DerefMut for GraphProcCtx<'a, 'g, 'p> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}
impl<'a, 'g, 'p> GraphProcCtx<'a, 'g, 'p> {
    pub fn new(ctx: &'a mut GraphCtx<'a, 'g>, proc_manager: &'p mut ProcManager) -> Self {
        ctx.as_proc_ctx(proc_manager)
    }
}
