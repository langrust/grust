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

impl Component {
    /// Store nodes applications as dependencies.
    pub fn add_node_dependencies(&self, graph: &mut DiGraphMap<usize, ()>) {
        match self {
            Component::Definition(comp_def) => comp_def.add_node_dependencies(graph),
            Component::Import(comp_import) => comp_import.add_node_dependencies(graph),
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
    pub fn compute_dependencies(&mut self, ctx: &mut Ctx) -> TRes<()> {
        match self {
            Component::Definition(comp_def) => comp_def.compute_dependencies(ctx),
            Component::Import(comp_import) => Ok(comp_import.compute_dependencies(ctx)),
        }
    }
}

impl ComponentImport {
    /// Store nodes applications as dependencies.
    pub fn add_node_dependencies(&self, graph: &mut DiGraphMap<usize, ()>) {
        // add [self] as node in graph
        graph.add_node(self.id);
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
    pub fn compute_dependencies(&mut self, ctx: &mut Ctx) {
        // initiate graph
        let mut graph = self.create_initialized_graph(ctx.symbol_table);

        // add output dependencies over inputs in graph
        ctx.symbol_table
            .get_node_outputs(self.id)
            .iter()
            .for_each(|(_, output)| {
                ctx.symbol_table
                    .get_node_inputs(self.id)
                    .into_iter()
                    .for_each(|input| {
                        add_edge(&mut graph, *output, *input, Label::Weight(0));
                    });
            });

        // set node's graph and reduced graph
        self.graph = graph.clone();
        ctx.reduced_graphs.insert(self.id, graph);
    }

    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    fn create_initialized_graph(&self, symbol_table: &SymbolTable) -> Graph {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // add input signals as vertices
        symbol_table
            .get_node_inputs(self.id)
            .into_iter()
            .for_each(|input| {
                graph.add_node(*input);
            });

        // return graph
        graph
    }
}

impl ComponentDefinition {
    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    fn create_initialized_graph(&self, symbol_table: &SymbolTable) -> Graph {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // add input signals as vertices
        symbol_table
            .get_node_inputs(self.id)
            .into_iter()
            .filter(|id| !symbol_table.get_type(**id).is_event()) // todo: is this important
            .for_each(|input| {
                graph.add_node(*input);
            });

        // add other signals as vertices
        for statement in &self.statements {
            let signals = statement.pattern.identifiers();
            signals.iter().for_each(|signal| {
                graph.add_node(*signal);
            });
        }

        // return graph
        graph
    }

    /// Create an initialized process manager from a node.
    fn create_initialized_process_manager(
        &self,
        symbol_table: &SymbolTable,
    ) -> HashMap<usize, Color> {
        // create an empty hash
        let mut hash = HashMap::new();

        // add input signals with white color (unprocessed)
        symbol_table
            .get_node_inputs(self.id)
            .into_iter()
            .filter(|id| !symbol_table.get_type(**id).is_event())
            .for_each(|input| {
                hash.insert(*input, Color::White);
            });

        // add other signals with white color (unprocessed)
        for statement in &self.statements {
            let signals = statement.get_identifiers();
            signals.iter().for_each(|signal| {
                hash.insert(*signal, Color::White);
            });
        }

        // return hash
        hash
    }

    /// Store nodes applications as dependencies.
    pub fn add_node_dependencies(&self, graph: &mut DiGraphMap<usize, ()>) {
        // add [self] as node in graph
        graph.add_node(self.id);
        // add [self]->[called_nodes] as edges in graph
        self.statements.iter().for_each(|statement| {
            statement
                .expr
                .get_called_nodes()
                .into_iter()
                .for_each(|id| {
                    graph.add_edge(self.id, id, ());
                })
        });
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
    pub fn compute_dependencies(&mut self, ctx: &mut Ctx) -> TRes<()> {
        // initiate graph
        let mut graph = self.create_initialized_graph(ctx.symbol_table);

        // complete contract dependency graphs
        self.add_contract_dependencies(&mut graph);

        // complete contract dependency graphs
        {
            let mut ctx = ctx.as_graph_ctx(&mut graph);
            self.add_equations_dependencies(&mut ctx)?;
        }

        // set node's graph
        self.graph = graph;

        // construct reduced graph
        self.construct_reduced_graph(ctx);

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
    fn add_equations_dependencies(&self, ctx: &mut GraphCtx) -> TRes<()> {
        let mut process_manager = self.create_initialized_process_manager(ctx.symbol_table);

        // scope for inner `ctx`
        {
            let mut ctx = ctx.as_proc_ctx(&mut process_manager);
            // add local and output signals dependencies
            for s in self.statements.iter() {
                s.add_dependencies(&mut ctx)?
            }
        }

        // add input signals dependencies
        ctx.symbol_table
            .get_node_inputs(self.id)
            .iter()
            .filter(|id| !ctx.symbol_table.get_type(**id).is_event()) // why?
            .for_each(|signal| {
                // get signal's color
                let color = process_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;
            });

        Ok(())
    }

    fn construct_reduced_graph(&mut self, ctx: &mut Ctx) {
        ctx.reduced_graphs
            .insert(self.id, self.create_initialized_graph(ctx.symbol_table));

        let mut process_manager = self.create_initialized_process_manager(ctx.symbol_table);

        // add output dependencies over inputs in reduced graph
        ctx.symbol_table
            .get_node_outputs(self.id)
            .iter()
            .for_each(|(_, output_signal)| {
                self.add_signal_dependencies_over_inputs(*output_signal, ctx, &mut process_manager)
            });

        // set node's reduced graph
        self.reduced_graph = ctx.reduced_graphs.get(&self.id).unwrap().clone();
    }

    /// Add dependencies to node's inputs of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x which depends on input i
    ///     x: int = i;     // depends on input i
    /// }
    /// ```
    fn add_signal_dependencies_over_inputs(
        &self,
        signal: usize,
        ctx: &mut Ctx,
        process_manager: &mut HashMap<usize, Color>,
    ) {
        let ComponentDefinition { id: node, .. } = self;

        // get signal's color
        let color = process_manager.get_mut(&signal).expect(&format!(
            "signal '{}' should be in process manager",
            ctx.symbol_table.get_name(signal)
        ));

        match color {
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                // for every neighbors, get inputs dependencies and add it as signal dependencies
                for (_, neighbor_id, label1) in self.graph.edges(signal) {
                    // tells if the neighbor is an input
                    let is_input = ctx
                        .symbol_table
                        .get_node_inputs(self.id)
                        .iter()
                        .any(|input| neighbor_id.eq(input));

                    if is_input {
                        // get node's reduced graph (borrow checker)
                        let reduced_graph = ctx.reduced_graphs.get_mut(node).unwrap();
                        // if input then add neighbor to reduced graph
                        add_edge(reduced_graph, signal, neighbor_id, label1.clone());
                        // and add its input dependencies (contract dependencies)
                        self.graph
                            .edges(neighbor_id)
                            .for_each(|(_, input_id, label2)| {
                                add_edge(reduced_graph, signal, input_id, label1.add(label2))
                            });
                    } else {
                        // else compute neighbor's inputs dependencies
                        self.add_signal_dependencies_over_inputs(neighbor_id, ctx, process_manager);

                        // get node's reduced graph (borrow checker)
                        let reduced_graph = ctx.reduced_graphs.get_mut(node).unwrap();
                        let neighbor_edges = reduced_graph
                            .edges(neighbor_id)
                            .map(|(_, input_id, label)| (input_id, label.clone()))
                            .collect::<Vec<_>>();

                        // add dependencies as graph's edges:
                        // s = e depends on i <=> s -> i
                        neighbor_edges.into_iter().for_each(|(input_id, label2)| {
                            add_edge(reduced_graph, signal, input_id, label1.add(&label2));
                        })
                    }
                }

                // get signal's color
                let color = process_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;
            }
            Color::Black | Color::Grey => (),
        }
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
    fn add_contract_dependencies(&self, graph: &mut DiGraphMap<usize, Label>) {
        // add edges to the graph
        // corresponding to dependencies in contract's terms
        self.contract.add_dependencies(graph);
    }
}

impl stream::ExprKind {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self) -> Vec<usize> {
        match &self {
            Self::Constant { .. } | Self::Identifier { .. } | Self::Enumeration { .. } => vec![],
            Self::Application { fun, inputs } => {
                let mut nodes = inputs
                    .iter()
                    .flat_map(|expression| expression.get_called_nodes())
                    .collect::<Vec<_>>();
                let mut other_nodes = fun.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Abstraction { expr, .. } | Self::UnOp { expr, .. } => expr.get_called_nodes(),
            Self::BinOp { lft, rgt, .. } => {
                let mut nodes = lft.get_called_nodes();
                let mut other_nodes = rgt.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::IfThenElse { cnd, thn, els } => {
                let mut nodes = cnd.get_called_nodes();
                let mut other_nodes = thn.get_called_nodes();
                nodes.append(&mut other_nodes);
                let mut other_nodes = els.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Structure { fields, .. } => fields
                .iter()
                .flat_map(|(_, expression)| expression.get_called_nodes())
                .collect::<Vec<_>>(),
            Self::Array { elements } => elements
                .iter()
                .flat_map(|expression| expression.get_called_nodes())
                .collect::<Vec<_>>(),
            Self::Tuple { elements } => elements
                .iter()
                .flat_map(|expression| expression.get_called_nodes())
                .collect::<Vec<_>>(),
            Self::Match { expr, arms } => {
                let mut nodes = expr.get_called_nodes();
                let mut other_nodes = arms
                    .iter()
                    .flat_map(|(_, bound, body, expr)| {
                        let mut nodes = vec![];
                        body.iter().for_each(|statement| {
                            let mut other_nodes = statement.expr.get_called_nodes();
                            nodes.append(&mut other_nodes);
                        });
                        let mut other_nodes = expr.get_called_nodes();
                        nodes.append(&mut other_nodes);
                        let mut other_nodes = bound
                            .as_ref()
                            .map_or(vec![], |expr| expr.get_called_nodes());
                        nodes.append(&mut other_nodes);
                        nodes
                    })
                    .collect::<Vec<_>>();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::FieldAccess { expr, .. } => expr.get_called_nodes(),
            Self::TupleElementAccess { expr, .. } => expr.get_called_nodes(),
            Self::Map { expr, fun } => {
                let mut nodes = expr.get_called_nodes();
                let mut other_nodes = fun.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Fold { array, init, fun } => {
                let mut nodes = array.get_called_nodes();
                let mut other_nodes = init.get_called_nodes();
                nodes.append(&mut other_nodes);
                let mut other_nodes = fun.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Sort { expr, fun } => {
                let mut nodes = expr.get_called_nodes();
                let mut other_nodes = fun.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Zip { arrays } => arrays
                .iter()
                .flat_map(|expr| expr.get_called_nodes())
                .collect::<Vec<_>>(),
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
            Identifier { id, .. } => Self::ident_deps(ctx.symbol_table, *id),
            Abstraction { .. } => Self::abstraction_deps(),
            Enumeration { .. } => Self::enumeration_deps(),
            UnOp { expr, .. } => Self::unop_deps(ctx, expr),
            BinOp { lft, rgt, .. } => Self::binop_deps(ctx, lft, rgt),
            IfThenElse { cnd, thn, els } => Self::ite_deps(ctx, cnd, thn, els),
            Application { fun, inputs, .. } => Self::fun_app_deps(ctx, fun, inputs),
            Structure { fields, .. } => Self::structure_deps(ctx, fields),
            Array { elements } => Self::array_deps(ctx, elements),
            Tuple { elements } => Self::tuple_deps(ctx, elements),
            Match { expr, arms } => Self::match_deps(&self, ctx, expr, arms),
            FieldAccess { expr, .. } => Self::field_access_deps(&self, ctx, expr),
            TupleElementAccess { expr, .. } => Self::tuple_access_deps(ctx, expr),
            Map { expr, .. } => Self::map_deps(ctx, expr),
            Fold { array, init, .. } => Self::fold_deps(ctx, array, init),
            Sort { expr, .. } => Self::sort_deps(&self, ctx, expr),
            Zip { arrays } => Self::zip_deps(ctx, arrays),
        }
    }
}

impl stream::ExprKind {
    /// Compute dependencies of an abstraction stream expression.
    fn abstraction_deps() -> TRes<Vec<(usize, Label)>> {
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
        inputs: &Vec<stream::Expr>,
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
    pub fn array_deps(
        ctx: &mut GraphProcCtx,
        elms: &Vec<stream::Expr>,
    ) -> TRes<Vec<(usize, Label)>> {
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
    fn field_access_deps(
        &self,
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
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
    pub fn ident_deps(symbol_table: &SymbolTable, id: usize) -> TRes<Vec<(usize, Label)>> {
        // identifier depends on called identifier with label weight of 0
        if symbol_table.is_function(id) {
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
        &self,
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
        arms: &Vec<(
            hir::Pattern,
            Option<stream::Expr>,
            Vec<stream::Stmt>,
            stream::Expr,
        )>,
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
    pub fn sort_deps(
        &self,
        ctx: &mut GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get sorted expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a structure stream expression.
    pub fn structure_deps(
        ctx: &mut GraphProcCtx,
        fields: &Vec<(usize, stream::Expr)>,
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

    /// Compute dependencies of an tuple stream expression.
    pub fn tuple_deps(
        ctx: &mut GraphProcCtx,
        elms: &Vec<stream::Expr>,
    ) -> TRes<Vec<(usize, Label)>> {
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
    pub fn zip_deps(
        ctx: &mut GraphProcCtx,
        arrays: &Vec<stream::Expr>,
    ) -> TRes<Vec<(usize, Label)>> {
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
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        let File { components, .. } = self;

        // initialize dictionary for reduced graphs
        let mut nodes_reduced_graphs = HashMap::new();

        // create graph of nodes
        let mut nodes_graph = DiGraphMap::new();
        components
            .iter()
            .for_each(|component| component.add_node_dependencies(&mut nodes_graph));

        // sort nodes according to their dependencies
        let sorted_nodes = toposort(&nodes_graph, None).map_err(|component| {
            let error = Error::NotCausalNode {
                node: symbol_table.get_name(component.node_id()).clone(),
                loc: self.loc.clone(),
            };
            errors.push(error);
            TerminationError
        })?;
        components.sort_by(|c1, c2| {
            let index1 = sorted_nodes
                .iter()
                .position(|id| *id == c1.get_id())
                .expect("should be in sorted list");
            let index2 = sorted_nodes
                .iter()
                .position(|id| *id == c2.get_id())
                .expect("should be in sorted list");

            Ord::cmp(&index2, &index1)
        });

        // ordered nodes complete their dependency graphs
        let mut ctx = Ctx::new(symbol_table, &mut nodes_reduced_graphs, errors);
        components
            .iter_mut()
            .map(|component| component.compute_dependencies(&mut ctx))
            .collect::<TRes<()>>()?;

        Ok(())
    }
}

impl hir::stream::Stmt {
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
        let hir::Stmt { expr, loc, .. } = self;

        // get signal's color
        let color = ctx
            .proc_manager
            .get_mut(&signal)
            .expect("signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                // compute and get dependencies
                if expr.dependencies.get().is_none() {
                    expr.compute_dependencies(ctx)?;
                }

                // add dependencies as graph's edges:
                // s = e depends on s' <=> s -> s'
                expr.get_dependencies().iter().for_each(|(id, label)| {
                    // if there was another edge, keep the most important label
                    add_edge(ctx.graph, signal, *id, label.clone())
                });

                // get signal's color
                let color = ctx
                    .proc_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;

                Ok(())
            }
            // if processing: error
            Color::Grey => {
                let error = Error::NotCausalSignal {
                    signal: ctx.symbol_table.get_name(signal).clone(),
                    loc: loc.clone(),
                };
                ctx.errors.push(error);
                Err(TerminationError)
            }
            // if processed: nothing to do
            Color::Black => Ok(()),
        }
    }
}

impl stream::Expr {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self) -> Vec<usize> {
        match &self.kind {
            stream::Kind::Expression { expr } => expr.get_called_nodes(),
            stream::Kind::SomeEvent { expr } | stream::Kind::RisingEdge { expr } => {
                expr.get_called_nodes()
            }
            stream::Kind::FollowedBy { .. } | stream::Kind::NoneEvent => vec![],
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                ..
            } => {
                let mut nodes = inputs
                    .iter()
                    .flat_map(|(_, expr)| expr.get_called_nodes())
                    .collect::<Vec<_>>();
                nodes.push(*called_node_id);
                nodes
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
            stream::Kind::FollowedBy { ref constant, id } => {
                // constant should not have dependencies
                constant.compute_dependencies(ctx)?;
                debug_assert!({ constant.get_dependencies().is_empty() });

                // dependencies with the memory delay
                self.dependencies.set(vec![(*id, Label::Weight(1))]);
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
                // function "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                self.dependencies.set(
                    inputs
                        .iter()
                        .map(|(input_id, input_expression)| {
                            // compute input expression dependencies
                            input_expression.compute_dependencies(ctx)?;

                            let symbol_table = ctx.symbol_table;
                            // get reduced graph (graph with only inputs/outputs signals)
                            let reduced_graph = ctx.reduced_graphs.get_mut(called_node_id).unwrap();

                            // for each node's output, get dependencies from output to inputs
                            let dependencies = symbol_table
                                .get_node_outputs(*called_node_id)
                                .iter()
                                .flat_map(|(_, output_signal)| {
                                    reduced_graph.edge_weight(*output_signal, *input_id).map_or(
                                        vec![],
                                        |label1| {
                                            input_expression
                                                .get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .map(|(id, label2)| (id, label1.add(&label2)))
                                                .collect()
                                        },
                                    )
                                })
                                .collect();

                            Ok(dependencies)
                        })
                        .collect::<TRes<Vec<Vec<(usize, Label)>>>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<(usize, Label)>>(),
                );

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
