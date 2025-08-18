//! Dependency graph.

use std::ops::Deref;

prelude! { graph::* }

pub type Graph = DiGraphMap<usize, Label>;
pub type ReducedGraph = HashMap<usize, Graph>;
pub type ProcManager = HashMap<usize, Color>;

pub struct DepCtx<'a> {
    pub ctx0: &'a ir0::Ctx,
    pub reduced_graphs: &'a mut ReducedGraph,
    pub errors: &'a mut Vec<Error>,
}
impl std::ops::Deref for DepCtx<'_> {
    type Target = ir0::Ctx;
    fn deref(&self) -> &Self::Target {
        self.ctx0
    }
}
mk_new! { impl{'a} DepCtx<'a> =>
    new {
        ctx0: &'a ir0::Ctx,
        reduced_graphs: &'a mut ReducedGraph,
        errors: &'a mut Vec<Error>,
    }
}
impl<'a> DepCtx<'a> {
    pub fn as_graph_ctx<'g>(&'g mut self, graph: &'g mut Graph) -> GraphCtx<'a, 'g> {
        GraphCtx { ctx1: self, graph }
    }
}

pub struct GraphCtx<'a, 'graph> {
    pub ctx1: &'graph mut DepCtx<'a>,
    pub graph: &'graph mut Graph,
}
impl<'a, 'g> GraphCtx<'a, 'g> {
    pub fn new(ctx1: &'a mut DepCtx<'a>, graph: &'g mut Graph) -> Self {
        Self { ctx1, graph }
    }
}
impl<'a> std::ops::Deref for GraphCtx<'a, '_> {
    type Target = DepCtx<'a>;
    fn deref(&self) -> &Self::Target {
        self.ctx1
    }
}
impl std::ops::DerefMut for GraphCtx<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx1
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
impl<'a, 'g> std::ops::Deref for GraphProcCtx<'a, 'g, '_> {
    type Target = GraphCtx<'a, 'g>;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
impl std::ops::DerefMut for GraphProcCtx<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}
impl<'a, 'g, 'p> GraphProcCtx<'a, 'g, 'p> {
    pub fn new(ctx: &'a mut GraphCtx<'a, 'g>, proc_manager: &'p mut ProcManager) -> Self {
        ctx.as_proc_ctx(proc_manager)
    }
}

impl Component {
    /// Store nodes applications as dependencies.
    pub fn add_node_dependencies(&self, graph: &mut DiGraphMap<usize, ()>) {
        // add [self] as node in graph
        graph.add_node(self.sign.id);
        match &self.body_or_path {
            Either::Left(body) => body.add_node_dependencies(self.sign.id, graph),
            Either::Right(_) => (),
        }
    }

    /// Compute the dependency graph of the node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int, j: int)
    /// requires { j < i }  // i and j depend on each other
    /// ensures  { j < o }  // o and j depend on each other
    /// { // i depends on nothing
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn compute_dependencies(&mut self, ctx: &mut DepCtx, mut stats: StatsMut) -> TRes<()> {
        match &mut self.body_or_path {
            Either::Left(body) => {
                // create a graph with inputs
                let mut graph = DiGraphMap::new();
                self.sign.inputs_in_graph(&mut graph, ctx);
                // create process manager with inputs
                let mut process_manager = HashMap::new();
                self.sign.inputs_in_proc(&mut process_manager, ctx);
                // add dependencies
                stats.timed(("compute dependencies (ir1)").to_string(), || {
                    body.compute_dependencies(graph, process_manager, ctx)
                })?;
                // construct reduced graph
                stats.timed_with(("construct reduced graph (ir1)").to_string(), |sub_stats| {
                    self.sign
                        .construct_reduced_graph(&body.graph, ctx, sub_stats)
                });
                Ok(())
            }
            Either::Right(_) => {
                // direct reduced graph (inputs -[0]-> outputs)
                stats.timed(("construct reduced graph (ir1)").to_string(), || {
                    self.sign.construct_reduced_graph_ext(ctx)
                });
                Ok(())
            }
        }
    }
}

impl ComponentSignature {
    /// Add inputs as vertices.
    fn inputs_in_graph(&self, graph: &mut Graph, ctx: &ir0::Ctx) {
        // add input signals as vertices
        ctx.get_node_inputs(self.id).iter().for_each(|input| {
            graph.add_node(*input);
        });
    }

    /// Add inputs in process manager.
    fn inputs_in_proc(&self, proc: &mut HashMap<usize, Color>, ctx: &ir0::Ctx) {
        // add input signals with white color (unprocessed)
        ctx.get_node_inputs(self.id).iter().for_each(|input| {
            proc.insert(*input, Color::White);
        });
    }

    /// Construct reduced graph, linking inputs to their dependent outputs.
    fn construct_reduced_graph(&mut self, graph: &Graph, ctx: &mut DepCtx, mut stats: StatsMut) {
        let mut reduced_graph = DiGraphMap::new();
        let inputs = ctx.get_node_inputs(self.id);
        let mut stack: Vec<usize> = inputs.clone();
        let mut seen: Vec<usize> = Vec::with_capacity(ctx.node_idents_number(self.id));

        // remove all local nodes from reduced_graph
        stats.timed("generate reduced graph", || {
            while let Some(to_propag) = stack.pop() {
                // set `seen`
                seen.push(to_propag);

                // get the idents that depends on `to_propag`
                let depending_edges = graph.edges_directed(to_propag, Direction::Incoming);
                // get the inputs on which `to_propag` depends
                let input_deps = reduced_graph
                    .edges_directed(to_propag, Direction::Outgoing)
                    .map(|(a, b, l): (usize, usize, &Label)| (a, b, *l))
                    .collect::<Vec<_>>();

                for (depending, _, l1) in depending_edges {
                    // add edge to inputs
                    if inputs.contains(&to_propag) {
                        add_edge(&mut reduced_graph, depending, to_propag, *l1)
                    } else {
                        for (_, input_id, l2) in input_deps.iter() {
                            add_edge(&mut reduced_graph, depending, *input_id, l1.add(l2))
                        }
                    }
                    // update stack: not already seen && no path with all the idents in stack
                    if !seen.contains(&depending)
                        && stack.iter().all(|id| {
                            !petgraph::algo::has_path_connecting(graph, depending, *id, None)
                        })
                    {
                        stack.push(depending);
                    }
                }
            }
        });

        // remove all local nodes from reduced_graph
        stats.timed("remove all local idents from reduced graph", || {
            ctx.get_node_locals(self.id).iter().for_each(|local_id| {
                reduced_graph.remove_node(**local_id);
            })
        });

        self.reduced_graph = reduced_graph;
        ctx.reduced_graphs
            .insert(self.id, self.reduced_graph.clone());
    }

    /// Construct reduced graph, linking inputs to their dependent outputs.
    fn construct_reduced_graph_ext(&mut self, ctx: &mut DepCtx) {
        let mut reduced_graph = DiGraphMap::new();

        // add output dependencies over inputs in reduced graph
        ctx.get_node_outputs(self.id)
            .iter()
            .zip(ctx.get_node_inputs(self.id))
            .for_each(|((_, output_id), input_id)| {
                add_edge(&mut reduced_graph, *output_id, *input_id, Label::Weight(0));
            });

        self.reduced_graph = reduced_graph;
        ctx.reduced_graphs
            .insert(self.id, self.reduced_graph.clone());
    }
}

impl ComponentBody {
    /// Add locals and outputs as vertices.
    fn defs_in_graph(&self, graph: &mut Graph) {
        // add other signals as vertices
        for statement in &self.statements {
            let signals = statement.get_identifiers();
            signals.into_iter().for_each(|signal| {
                graph.add_node(signal);
            });
        }
    }

    /// Add locals and outputs in process manager.
    fn defs_in_proc(&self, proc: &mut HashMap<usize, Color>) {
        // add other signals as vertices
        for statement in &self.statements {
            let signals = statement.get_identifiers();
            signals.into_iter().for_each(|signal| {
                proc.insert(signal, Color::White);
            });
        }
    }

    /// Store nodes applications as dependencies.
    pub fn add_node_dependencies(&self, comp_id: usize, graph: &mut DiGraphMap<usize, ()>) {
        // add [self]->[called_nodes] as edges in graph
        let mut nodes = Vec::with_capacity(29);
        for stmt in self.statements.iter() {
            debug_assert!(nodes.is_empty());
            stmt.expr.get_called_nodes(&mut nodes);
            for id in nodes.drain(0..) {
                graph.add_edge(comp_id, id, ());
            }
        }
    }

    /// Compute the dependency graph of the node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int, j: int)
    /// requires { j < i }  // i and j depend on each other
    /// ensures  { j < o }  // o and j depend on each other
    /// { // i depends on nothing
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn compute_dependencies(
        &mut self,
        mut graph: Graph,
        mut proc: HashMap<usize, Color>,
        ctx: &mut DepCtx,
    ) -> TRes<()> {
        // initiate graph and proc with locals and outputs
        self.defs_in_graph(&mut graph);
        self.defs_in_proc(&mut proc);

        // complete contract dependency graphs
        self.add_contract_dependencies(&mut graph, ctx);

        // complete dependency graph with equations
        {
            let mut ctx = ctx.as_graph_ctx(&mut graph);
            self.add_equations_dependencies(&mut proc, &mut ctx)?;
        }

        // set node's graph
        self.graph = graph;

        Ok(())
    }

    /// Complete dependency graph of the node's equations.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) { // i depends on nothing
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    fn add_equations_dependencies(
        &self,
        proc: &mut HashMap<usize, Color>,
        ctx: &mut GraphCtx,
    ) -> TRes<()> {
        // scope for inner `ctx`
        {
            let mut ctx = ctx.as_proc_ctx(proc);
            // add local and output signals dependencies
            for s in self.statements.iter() {
                s.add_dependencies(&mut ctx)?
            }
        }

        Ok(())
    }

    /// Add signal dependencies in contract.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int, j: int)
    /// requires { j < i }  // i and j depend on each other
    /// ensures  { j < o }  // o and j depend on each other
    /// {
    ///     out o: int = i;
    /// }
    /// ```
    fn add_contract_dependencies(&self, graph: &mut DiGraphMap<usize, Label>, ctx: &mut DepCtx) {
        // add edges to the graph
        // corresponding to dependencies in contract's terms
        self.contract.add_dependencies(graph, ctx.deref());
    }
}

impl stream::ExprKind {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self, target: &mut Vec<usize>) {
        match &self {
            Self::Constant { .. }
            | Self::Identifier { .. }
            | Self::Enumeration { .. }
            | Self::Lambda { .. } => (),
            Self::Application { fun, inputs } => {
                inputs.iter().for_each(|e| e.get_called_nodes(target));
                fun.get_called_nodes(target);
            }
            Self::UnOp { expr, .. } => expr.get_called_nodes(target),
            Self::BinOp { lft, rgt, .. } => {
                lft.get_called_nodes(target);
                rgt.get_called_nodes(target);
            }
            Self::IfThenElse { cnd, thn, els } => {
                cnd.get_called_nodes(target);
                thn.get_called_nodes(target);
                els.get_called_nodes(target);
            }
            Self::Structure { fields, .. } => fields
                .iter()
                .for_each(|(_, expression)| expression.get_called_nodes(target)),
            Self::Array { elements } => elements
                .iter()
                .for_each(|expression| expression.get_called_nodes(target)),
            Self::Tuple { elements } => elements
                .iter()
                .for_each(|expression| expression.get_called_nodes(target)),
            Self::MatchExpr { expr, arms } => {
                expr.get_called_nodes(target);
                for (_, bound, body, expr) in arms.iter() {
                    for stmt in body.iter() {
                        stmt.expr.get_called_nodes(target);
                    }
                    expr.get_called_nodes(target);
                    if let Some(bound) = bound.as_ref() {
                        bound.get_called_nodes(target);
                    }
                }
            }
            Self::FieldAccess { expr, .. }
            | Self::TupleElementAccess { expr, .. }
            | Self::ArrayAccess { expr, .. } => expr.get_called_nodes(target),
            Self::Map { expr, fun } => {
                expr.get_called_nodes(target);
                fun.get_called_nodes(target);
            }
            Self::Fold { array, init, fun } => {
                array.get_called_nodes(target);
                init.get_called_nodes(target);
                fun.get_called_nodes(target);
            }
            Self::Sort { expr, fun } => {
                expr.get_called_nodes(target);
                fun.get_called_nodes(target);
            }
            Self::Zip { arrays } => arrays.iter().for_each(|expr| expr.get_called_nodes(target)),
        }
    }

    /// Compute dependencies of a stream expression.
    ///
    /// # Example
    ///
    /// Considering the following node:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o: int = 0 fby z;
    ///     z: int = 1 fby (x + y);
    /// }
    /// ```
    ///
    /// The stream expression `my_node(f(x), 1).o` depends on the signal `x` with
    /// a dependency label weight of 2. Indeed, the expression depends on the memory
    /// of the memory of `x` (the signal is behind 2 fby operations).
    pub fn compute_dependencies(&self, ctx: &mut GraphProcCtx) -> TRes<Vec<(usize, Label)>> {
        use expr::Kind::*;
        match self {
            Constant { .. } => Self::constant_deps(),
            Identifier { id, .. } => Self::ident_deps(ctx.ctx, *id),
            Lambda { .. } => Self::lambda_deps(),
            Enumeration { .. } => Self::enumeration_deps(),
            UnOp { expr, .. } => Self::unop_deps(ctx, expr),
            BinOp { lft, rgt, .. } => Self::binop_deps(ctx, lft, rgt),
            IfThenElse { cnd, thn, els } => Self::ite_deps(ctx, cnd, thn, els),
            Application { fun, inputs, .. } => Self::fun_app_deps(ctx, fun, inputs),
            Structure { fields, .. } => Self::structure_deps(ctx, fields),
            Array { elements } => Self::array_deps(ctx, elements),
            Tuple { elements } => Self::tuple_deps(ctx, elements),
            MatchExpr { expr, arms } => Self::match_deps(ctx, expr, arms),
            FieldAccess { expr, .. } => Self::field_access_deps(ctx, expr),
            TupleElementAccess { expr, .. } => Self::tuple_access_deps(ctx, expr),
            ArrayAccess { expr, .. } => Self::array_access_deps(ctx, expr),
            Map { expr, .. } => Self::map_deps(ctx, expr),
            Fold { array, init, .. } => Self::fold_deps(ctx, array, init),
            Sort { expr, .. } => Self::sort_deps(ctx, expr),
            Zip { arrays } => Self::zip_deps(ctx, arrays),
        }
    }
}

impl stream::ExprKind {
    /// Compute dependencies of a lambda stream expression.
    fn lambda_deps() -> TRes<Vec<(usize, Label)>> {
        Ok(vec![])
    }

    /// Compute dependencies of a constant stream expression.
    fn constant_deps() -> TRes<Vec<(usize, Label)>> {
        Ok(vec![])
    }

    /// Compute dependencies of an enumeration stream expression.
    fn enumeration_deps() -> TRes<Vec<(usize, Label)>> {
        Ok(vec![])
    }

    fn fun_app_deps(
        ctx: &mut GraphProcCtx,
        function: &stream::Expr,
        inputs: &[stream::Expr],
    ) -> TRes<Vec<(usize, Label)>> {
        // propagate dependencies computation
        function.compute_dependencies(ctx)?;
        // retrieve deps to augment with inputs
        let mut dependencies = function.get_dependencies().clone();

        for i in inputs.iter() {
            i.compute_dependencies(ctx)?;
            dependencies.extend(i.get_dependencies().iter().cloned());
        }

        Ok(dependencies)
    }

    /// Compute dependencies of an array stream expression.
    pub fn array_deps(ctx: &mut GraphProcCtx, elms: &[stream::Expr]) -> TRes<Vec<(usize, Label)>> {
        let mut res = Vec::with_capacity(elms.len());
        // propagate dependencies computation
        for e in elms.iter() {
            e.compute_dependencies(ctx)?;
            res.extend(e.get_dependencies().iter().cloned());
        }
        Ok(res)
    }

    /// Compute dependencies of a binop stream expression.
    fn binop_deps(
        ctx: &mut GraphProcCtx,
        lhs: &stream::Expr,
        rhs: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get right and left expressions dependencies
        lhs.compute_dependencies(ctx)?;
        rhs.compute_dependencies(ctx)?;
        let mut deps = lhs.get_dependencies().clone();
        deps.extend(rhs.get_dependencies().iter().cloned());

        Ok(deps)
    }

    /// Compute dependencies of a field access stream expression.
    fn field_access_deps(ctx: &mut GraphProcCtx, expr: &stream::Expr) -> TRes<Vec<(usize, Label)>> {
        // get accessed expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a fold stream expression.
    fn fold_deps(
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
        init: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get folded expression dependencies
        expr.compute_dependencies(ctx)?;
        let mut deps = expr.get_dependencies().clone();

        // get initialization expression dependencies
        init.compute_dependencies(ctx)?;
        deps.extend(init.get_dependencies().iter().cloned());

        Ok(deps)
    }

    /// Compute dependencies of an identifier.
    pub fn ident_deps(ctx: &Ctx, id: usize) -> TRes<Vec<(usize, Label)>> {
        // identifier depends on called identifier with label weight of 0
        if ctx.is_function(id) {
            Ok(vec![])
        } else {
            Ok(vec![(id, Label::Weight(0))])
        }
    }

    /// Compute dependencies of a if-then-else stream expression.
    pub fn ite_deps(
        ctx: &mut GraphProcCtx,
        cnd: &stream::Expr,
        thn: &stream::Expr,
        els: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // dependencies of if-then-else are dependencies of the expressions
        cnd.compute_dependencies(ctx)?;
        thn.compute_dependencies(ctx)?;
        els.compute_dependencies(ctx)?;

        let mut deps = cnd.get_dependencies().clone();
        deps.extend(thn.get_dependencies().iter().cloned());
        deps.extend(els.get_dependencies().iter().cloned());

        Ok(deps)
    }

    /// Compute dependencies of a map stream expression.
    pub fn map_deps(ctx: &mut GraphProcCtx, expr: &stream::Expr) -> TRes<Vec<(usize, Label)>> {
        // get mapped expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a match stream expression.
    pub fn match_deps(
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
        arms: &[(
            ir1::Pattern,
            Option<stream::Expr>,
            Vec<stream::Stmt>,
            stream::Expr,
        )],
    ) -> TRes<Vec<(usize, Label)>> {
        // compute arms dependencies
        let mut deps = Vec::with_capacity(25);

        for (pattern, bound, body, arm_expression) in arms.iter() {
            // get local signals defined in pattern
            let local_signals = pattern.identifiers();
            // extends `deps` with the input iterator without local signals
            macro_rules! add_deps {
                    {$iter:expr} => {
                        deps.extend(
                            $iter.filter(|(signal, _)| !local_signals.contains(signal)).cloned()
                        )
                    }
                }

            for statement in body {
                statement.add_dependencies(ctx)?;
                add_deps!(statement.expr.get_dependencies().iter());
            }

            // get arm expression dependencies
            arm_expression.compute_dependencies(ctx)?;
            add_deps!(arm_expression.get_dependencies().iter());

            // get bound dependencies
            if let Some(expr) = bound {
                expr.compute_dependencies(ctx)?;
                add_deps!(expr.get_dependencies().iter());
            }
        }

        // get matched expression dependencies
        expr.compute_dependencies(ctx)?;
        deps.extend(expr.get_dependencies().iter().cloned());

        Ok(deps)
    }

    /// Compute dependencies of a sort stream expression.
    pub fn sort_deps(ctx: &mut GraphProcCtx, expr: &stream::Expr) -> TRes<Vec<(usize, Label)>> {
        // get sorted expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a structure stream expression.
    pub fn structure_deps(
        ctx: &mut GraphProcCtx,
        fields: &[(usize, stream::Expr)],
    ) -> TRes<Vec<(usize, Label)>> {
        // propagate dependencies computation
        let mut deps = Vec::with_capacity(25);
        for (_, expr) in fields {
            expr.compute_dependencies(ctx)?;
            deps.extend(expr.get_dependencies().iter().cloned())
        }
        // not shrinking, this might grow later
        Ok(deps)
    }

    /// Compute dependencies of a tuple element access stream expression.
    pub fn tuple_access_deps(
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get accessed expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of an array element access stream expression.
    pub fn array_access_deps(
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get accessed expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of an tuple stream expression.
    pub fn tuple_deps(ctx: &mut GraphProcCtx, elms: &[stream::Expr]) -> TRes<Vec<(usize, Label)>> {
        let mut deps = Vec::with_capacity(25);
        // propagate dependencies computation
        for e in elms.iter() {
            e.compute_dependencies(ctx)?;
            deps.extend(e.get_dependencies().iter().cloned())
        }
        // not shrinking, this might grow later
        Ok(deps)
    }

    /// Compute dependencies of a unop stream expression.
    pub fn unop_deps(ctx: &mut GraphProcCtx, expr: &stream::Expr) -> TRes<Vec<(usize, Label)>> {
        // get expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a zip stream expression.
    pub fn zip_deps(ctx: &mut GraphProcCtx, arrays: &[stream::Expr]) -> TRes<Vec<(usize, Label)>> {
        let mut deps = Vec::with_capacity(25);
        // propagate dependencies computation
        for a in arrays.iter() {
            a.compute_dependencies(ctx)?;
            deps.extend(a.get_dependencies().iter().cloned());
        }
        Ok(deps)
    }
}

impl File {
    /// Generate dependency graph for every nodes/component.
    pub fn generate_dependency_graphs(
        &mut self,
        ctx: &mut ir0::Ctx,
        mut stats: StatsMut,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        // initialize dictionary for reduced graphs
        let mut nodes_reduced_graphs = HashMap::new();

        // create graph of nodes
        let mut nodes_graph = DiGraphMap::new();
        stats.timed("component dependencies graph (ir1)", || {
            self.components
                .iter()
                .for_each(|component| component.add_node_dependencies(&mut nodes_graph))
        });

        // sort nodes according to their dependencies
        let sorted_nodes = stats.timed("toposort of component dependencies graph (ir1)", || {
            toposort(&nodes_graph, None)
                .map_err(|component| {
                    error!(@self.loc =>
                        ErrorKind::node_non_causal(
                            ctx.get_name(component.node_id()).to_string()
                        )
                    )
                })
                .dewrap(errors)
        })?;

        stats.timed("sort components using toposort (ir1)", || {
            self.components.sort_by(|c1, c2| {
                let index1 = sorted_nodes
                    .iter()
                    .position(|id| *id == c1.get_id())
                    .expect("internal error: should be in sorted list");
                let index2 = sorted_nodes
                    .iter()
                    .position(|id| *id == c2.get_id())
                    .expect("internal error: should be in sorted list");
                Ord::cmp(&index2, &index1)
            })
        });

        // ordered nodes complete their dependency graphs
        let mut ctx = DepCtx::new(ctx, &mut nodes_reduced_graphs, errors);
        self.components
            .iter_mut()
            .map(|component| {
                stats.timed_with(
                    format!(
                        "`{}` dependency graph generation (ir1)",
                        ctx.get_name(component.get_id())
                    ),
                    |sub_stats| component.compute_dependencies(&mut ctx, sub_stats),
                )
            })
            .collect_res()?;

        Ok(())
    }
}

impl ir1::stream::Stmt {
    /// Add direct dependencies of a statement.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_dependencies(&self, ctx: &mut GraphProcCtx) -> TRes<()> {
        let signals = self.pattern.identifiers();
        for signal in signals {
            self.add_signal_dependencies(signal, ctx)?
        }
        Ok(())
    }

    /// Add direct dependencies of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_signal_dependencies(&self, signal: usize, ctx: &mut GraphProcCtx) -> TRes<()> {
        // get signal's color
        let color = ctx
            .proc_manager
            .get_mut(&signal)
            .expect("internal error: signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                // compute and get dependencies
                if self.expr.dependencies.get().is_none() {
                    self.expr.compute_dependencies(ctx)?;
                }

                // add dependencies as graph's edges:
                // s = e depends on s' <=> s -> s'
                self.expr.get_dependencies().iter().for_each(|(id, label)| {
                    // if there was another edge, keep the most important label
                    add_edge(ctx.graph, signal, *id, *label)
                });

                // get signal's color
                let color = ctx
                    .proc_manager
                    .get_mut(&signal)
                    .expect("internal error: signal should be in processing manager");
                // update status: processed
                *color = Color::Black;

                Ok(())
            }
            // if processing: error
            Color::Grey => {
                let name = ctx.ctx.get_name(signal).clone();
                bad!(ctx.errors, @self.loc => ErrorKind::signal_non_causal(name.to_string()))
            }
            // if processed: nothing to do
            Color::Black => Ok(()),
        }
    }
}

impl stream::Expr {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self, target: &mut Vec<usize>) {
        match &self.kind {
            stream::Kind::Expression { expr } => expr.get_called_nodes(target),
            stream::Kind::SomeEvent { expr } | stream::Kind::RisingEdge { expr } => {
                expr.get_called_nodes(target)
            }
            stream::Kind::Last { .. } | stream::Kind::NoneEvent => (),
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                ..
            } => {
                inputs
                    .iter()
                    .for_each(|(_, expr)| expr.get_called_nodes(target));
                target.push(*called_node_id);
            }
        }
    }

    /// Compute dependencies of a stream expression.
    ///
    /// # Example
    ///
    /// Considering the following node:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o: int = 0 fby z;
    ///     z: int = 1 fby (x + y);
    /// }
    /// ```
    ///
    /// The stream expression `my_node(f(x), 1).o` depends on the signal `x` with
    /// a dependency label weight of 2. Indeed, the expression depends on the memory
    /// of the memory of `x` (the signal is behind 2 fby operations).
    pub fn compute_dependencies(&self, ctx: &mut GraphProcCtx) -> TRes<()> {
        match &self.kind {
            stream::Kind::Last { signal_id, .. } => {
                // dependencies with the memory delay
                self.dependencies.set(vec![(*signal_id, Label::Weight(1))]);
                Ok(())
            }
            stream::Kind::RisingEdge { ref expr } => {
                // propagate dependencies computation in expression
                expr.compute_dependencies(ctx)?;
                // dependencies with the memory delay
                let mut dependencies = expr
                    .get_dependencies()
                    .iter()
                    .map(|(id, label)| (*id, label.increment()))
                    .collect::<Vec<_>>();
                // rising edge depends on current value and memory
                dependencies.extend(expr.get_dependencies());
                self.dependencies.set(dependencies);
                Ok(())
            }
            stream::Kind::NodeApplication {
                ref called_node_id,
                ref inputs,
                ..
            } => {
                let deps = {
                    let mut vec = Vec::with_capacity(inputs.len() * 3);
                    let mut res = Ok(());
                    macro_rules! handle {
                        { $e:expr } => {
                            match $e {
                                Ok(val) => val,
                                Err(e) => if res.is_ok() { res = Err(e) }
                            }
                        }
                    }
                    for (input_id, input_expression) in inputs.iter() {
                        // compute input expression dependencies
                        handle!(input_expression.compute_dependencies(ctx));

                        let DepCtx {
                            ctx0,
                            ref mut reduced_graphs,
                            ..
                        } = ctx.ctx1;

                        // get reduced graph (graph with only inputs/outputs signals)
                        let reduced_graph = reduced_graphs.get_mut(called_node_id).unwrap();

                        // for each node's output, get dependencies from output to inputs
                        for (_, output_signal) in ctx0.get_node_outputs(*called_node_id).iter() {
                            if let Some(label1) =
                                reduced_graph.edge_weight(*output_signal, *input_id)
                            {
                                for (id, label2) in input_expression.get_dependencies().iter() {
                                    vec.push((*id, label1.add(label2)));
                                }
                            }
                        }
                    }
                    res?;
                    vec
                };
                // function "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                self.dependencies.set(deps);
                Ok(())
            }
            stream::Kind::Expression { expr } => {
                self.dependencies.set(expr.compute_dependencies(ctx)?);
                Ok(())
            }
            stream::Kind::SomeEvent { expr } => {
                // propagate dependencies computation in expression
                expr.compute_dependencies(ctx)?;
                self.dependencies.set(expr.get_dependencies().clone());
                Ok(())
            }
            stream::Kind::NoneEvent => {
                // no dependencies
                self.dependencies.set(vec![]);
                Ok(())
            }
        }
    }
}
