//! [Component] module.

prelude! {
    graph::*,
}

use super::memory::Memory;

#[derive(Debug, Clone, PartialEq)]
/// LanGRust component.
pub struct Component {
    /// Component's signature
    pub sign: ComponentSignature,
    /// Component's
    pub body_or_path: Either<ComponentBody, syn::Path>,
}
impl Component {
    pub fn new(
        id: usize,
        inits: Vec<ir1::stream::InitStmt>,
        statements: Vec<ir1::stream::Stmt>,
        contract: ir1::Contract,
        loc: Loc,
    ) -> Self {
        Self {
            sign: ComponentSignature::new(id, loc),
            body_or_path: Either::Left(ComponentBody::new(inits, statements, contract)),
        }
    }

    pub fn new_ext(id: usize, path: syn::Path, loc: Loc) -> Self {
        Self {
            sign: ComponentSignature::new(id, loc),
            body_or_path: Either::Right(path),
        }
    }

    pub fn get_graph(&self) -> Option<&DiGraphMap<usize, Label>> {
        match &self.body_or_path {
            Either::Left(body) => Some(&body.graph),
            Either::Right(_) => None,
        }
    }
    pub fn get_reduced_graph(&self) -> &DiGraphMap<usize, Label> {
        &self.sign.reduced_graph
    }
    pub fn get_id(&self) -> usize {
        self.sign.id
    }
    pub fn get_location(&self) -> Loc {
        self.sign.loc
    }

    /// Check the causality of the component.
    ///
    /// # Example
    ///
    /// The following simple component is causal, there is no causality loop.
    ///
    /// ```GR
    /// component causal_node1(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    /// ```
    ///
    /// The next component is causal as well, `x` does not depends on `o` but depends on the memory of
    /// `o`. Then there is no causality loop.
    ///
    /// ```GR
    /// component causal_node2() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the component that follows is not causal, `o` depends on `x` which depends on `o`. Values of
    /// signals can not be determined, then the compilation raises a causality error.
    ///
    /// ```GR
    /// component not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    /// ```
    pub fn causal(&self, symbol_table: &Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        match &self.body_or_path {
            Either::Left(body) => {
                // construct component's subgraph containing only 0-label weight
                let mut subgraph = body.graph.clone();
                body.graph
                    .all_edges()
                    .for_each(|(from, to, label)| match label {
                        Label::Weight(0) => (),
                        _ => {
                            let _label = subgraph.remove_edge(from, to);
                            debug_assert_ne!(_label, Some(Label::Weight(0)))
                        }
                    });

                // if a schedule exists, then the component is causal
                let res = graph::toposort(&subgraph, None);
                if let Err(signal) = res {
                    let name = symbol_table.get_name(signal.node_id());
                    bad!( errors, @self.get_location() => ErrorKind::signal_non_causal(name.to_string()) )
                }

                Ok(())
            }
            Either::Right(_) => Ok(()),
        }
    }

    /// Create memory for [ir1] component.
    ///
    /// Store buffer for followed by expressions and component applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// component test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// component test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         mem_my_node_o_: (my_node, o);
    ///     },
    /// }
    /// ```
    pub fn memorize(&mut self, symbol_table: &mut Ctx) -> Res<()> {
        match &mut self.body_or_path {
            Either::Left(body) => body.memorize(symbol_table),
            Either::Right(_) => Ok(()),
        }
    }

    /// Change [ir1] component into a normal form.
    ///
    /// The normal form of a component is as follows:
    /// - component application can only append at root expression
    /// - component application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// component test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above component contains the following components:
    ///
    /// ```GR
    /// component test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// component test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are transformed into:
    ///
    /// ```GR
    /// component test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// component test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        symbol_table: &mut Ctx,
    ) {
        match &mut self.body_or_path {
            Either::Left(body) => body.normal_form(nodes_reduced_graphs, symbol_table),
            Either::Right(_) => (),
        }
    }

    /// Inline component application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// component semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// component fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal. There is
    /// no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`, which can
    /// not be computed by a function call.
    pub fn inline_when_needed(
        &mut self,
        components: &HashMap<usize, Component>,
        symbol_table: &mut Ctx,
    ) {
        match &mut self.body_or_path {
            Either::Left(body) => body.inline_when_needed(components, symbol_table),
            Either::Right(_) => (),
        }
    }

    /// Instantiate component's statements with inputs.
    ///
    /// It will return new statements where the input signals are instantiated by expressions. New
    /// statements should have fresh id according to the calling component.
    ///
    /// # Example
    ///
    /// ```GR
    /// component to_be_inlined(i: int) {
    ///     o: int = 0 fby j;
    ///     out j: int = i + 1;
    /// }
    ///
    /// component calling_node(i: int) {
    ///     out o: int = to_be_inlined(o);
    ///     j: int = i * o;
    /// }
    /// ```
    ///
    /// The call to `to_be_inlined` will generate th following statements:
    ///
    /// ```GR
    /// o: int = 0 fby j_1;
    /// j_1: int = o + 1;
    /// ```
    pub fn instantiate_statements_and_memory(
        &self,
        identifier_creator: &mut IdentifierCreator,
        inputs: &[(usize, stream::Expr)],
        new_output_pattern: Option<stmt::Pattern>,
        symbol_table: &mut Ctx,
    ) -> (Vec<stream::Stmt>, Memory) {
        match &self.body_or_path {
            Either::Left(body) => {
                // create the context with the given inputs
                let mut context_map = inputs
                    .iter()
                    .map(|(input, expression)| (*input, Either::Right(expression.clone())))
                    .collect::<HashMap<_, _>>();

                // add output signals to context
                new_output_pattern.map(|pattern| {
                    let signals = pattern.identifiers();
                    symbol_table
                        .get_node_outputs(self.sign.id)
                        .iter()
                        .zip(signals)
                        .for_each(|((_, output_id), new_output_id)| {
                            context_map.insert(*output_id, Either::Left(new_output_id));
                        })
                });

                body.instantiate_statements_and_memory(
                    identifier_creator,
                    context_map,
                    symbol_table,
                )
            }
            Either::Right(_) => (vec![], Memory::new()),
        }
    }

    /// Schedule statements.
    ///
    /// # Example.
    ///
    /// ```GR
    /// component test(v: int) {
    ///     out y: int = x-1
    ///     o_1: int = 0 fby x
    ///     x: int = v*2 + o_1
    /// }
    /// ```
    ///
    /// In the component above, signal `y` depends on the current value of `x`, `o_1` depends on the
    /// memory of `x` and `x` depends on `v` and `o_1`. The component is causal and should be scheduled
    /// as bellow:
    ///
    /// ```GR
    /// component test(v: int) {
    ///     o_1: int = 0 fby x  // depends on no current values of signals
    ///     x: int = v*2 + o_1  // depends on the computed value of `o_1` and given `v`
    ///     out y: int = x-1    // depends on the computed value of `x`
    /// }
    /// ```
    pub fn schedule(&mut self) {
        match &mut self.body_or_path {
            Either::Left(body) => body.schedule(),
            Either::Right(_) => (),
        }
    }

    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        match &self.body_or_path {
            Either::Left(body) => body.is_normal_form(),
            Either::Right(_) => true,
        }
    }

    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        match &self.body_or_path {
            Either::Left(body) => body.no_component_application(),
            Either::Right(_) => true,
        }
    }
}

#[derive(Debug, Clone)]
/// LanGRust component definition.
pub struct ComponentBody {
    /// Component's initialization statements.
    pub inits: Vec<ir1::stream::InitStmt>,
    /// Component's statements.
    pub statements: Vec<ir1::stream::Stmt>,
    /// Component's contract.
    pub contract: ir1::Contract,
    /// Unitary component's memory.
    pub memory: Memory,
    /// Component dependency graph.
    pub graph: DiGraphMap<usize, Label>,
}

impl PartialEq for ComponentBody {
    fn eq(&self, other: &Self) -> bool {
        self.inits == other.inits
            && self.statements == other.statements
            && self.contract == other.contract
            && self.eq_graph(other)
    }
}

impl ComponentBody {
    pub fn new(
        inits: Vec<ir1::stream::InitStmt>,
        statements: Vec<ir1::stream::Stmt>,
        contract: ir1::Contract,
    ) -> Self {
        Self {
            inits,
            statements,
            contract,
            graph: graph::DiGraphMap::new(),
            memory: ir1::Memory::new(),
        }
    }

    /// Return vector of component's signals id.
    pub fn get_signals_id(&self) -> Vec<usize> {
        self.statements
            .iter()
            .flat_map(|statement| statement.get_identifiers())
            .collect()
    }

    /// Return vector of component's signals name.
    pub fn get_signals_names(&self, symbol_table: &Ctx) -> Vec<Ident> {
        self.statements
            .iter()
            .flat_map(|statement| statement.get_identifiers())
            .chain(self.memory.get_identifiers().cloned())
            .map(|id| symbol_table.get_name(id).clone())
            .collect()
    }

    fn eq_graph(&self, other: &ComponentBody) -> bool {
        let graph_nodes = self.graph.nodes();
        let other_nodes = other.graph.nodes();
        let graph_edges = self.graph.all_edges();
        let other_edges = other.graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }

    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.is_normal_form())
    }
    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        self.statements
            .iter()
            .all(|statement| statement.no_component_application())
    }

    /// Create memory for [ir1] component.
    ///
    /// Store buffer for followed by expressions and component applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// component test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// component test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         mem_my_node_o_: (my_node, o);
    ///     },
    /// }
    /// ```
    pub fn memorize(&mut self, symbol_table: &mut Ctx) -> Res<()> {
        // create an IdentifierCreator, a local Ctx and Memory
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_names(symbol_table));
        symbol_table.local();
        let mut memory = Memory::new();

        for init in self.inits.drain(..) {
            init.memorize(&mut memory, symbol_table)?;
        }

        for statement in self.statements.iter_mut() {
            statement.memorize(&mut identifier_creator, &mut memory, symbol_table)?;
        }

        self.contract
            .memorize(&mut identifier_creator, &mut memory, symbol_table);

        // drop IdentifierCreator (auto), local Ctx and set Memory
        symbol_table.global();
        self.memory = memory;

        // add a dependency graph to the component
        let mut graph = GraphMap::new();
        self.get_signals_id().iter().for_each(|signal_id| {
            graph.add_node(*signal_id);
        });
        self.statements
            .iter()
            .for_each(|statement| statement.add_to_graph(&mut graph));
        self.graph = graph;
        Ok(())
    }

    /// Change [ir1] component into a normal form.
    ///
    /// The normal form of a component is as follows:
    /// - component application can only append at root expression
    /// - component application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// component test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above component contains the following components:
    ///
    /// ```GR
    /// component test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// component test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are transformed into:
    ///
    /// ```GR
    /// component test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// component test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        symbol_table: &mut Ctx,
    ) {
        // create an IdentifierCreator and a local Ctx
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_names(symbol_table));
        symbol_table.local();

        for init in &self.inits {
            // initialization expressions should already be in normal form
            debug_assert!(init.expr.is_normal_form());
        }

        let mut new_stmts = vec![];
        for stmt in self.statements.drain(..) {
            let (add_stmts, add_inits) =
                stmt.normal_form(nodes_reduced_graphs, &mut identifier_creator, symbol_table);
            new_stmts.extend(add_stmts);
            self.inits.extend(add_inits);
        }
        self.statements = new_stmts;

        // drop IdentifierCreator (auto) and local Ctx
        symbol_table.global();

        // add a dependency graph to the component
        let mut graph = GraphMap::new();
        self.get_signals_id().iter().for_each(|signal_id| {
            graph.add_node(*signal_id);
        });
        self.statements
            .iter()
            .for_each(|statement| statement.add_to_graph(&mut graph));
        self.graph = graph;
    }

    /// Inline component application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// component semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// component fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal. There is
    /// no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`, which can
    /// not be computed by a function call.
    pub fn inline_when_needed(
        &mut self,
        components: &HashMap<usize, Component>,
        symbol_table: &mut Ctx,
    ) {
        // create identifier creator containing the signals
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_names(symbol_table));

        // compute new statements for the component
        let mut new_statements: Vec<stream::Stmt> = vec![];
        std::mem::take(&mut self.statements)
            .into_iter()
            .for_each(|statement| {
                let mut retrieved_statements = statement.inline_when_needed_recursive(
                    &mut self.memory,
                    &mut identifier_creator,
                    symbol_table,
                    components,
                );
                new_statements.append(&mut retrieved_statements)
            });

        // update component's stmt
        self.update_statements(&new_statements)
    }

    /// Instantiate component's statements with inputs.
    ///
    /// It will return new statements where the input signals are instantiated by expressions. New
    /// statements should have fresh id according to the calling component.
    ///
    /// # Example
    ///
    /// ```GR
    /// component to_be_inlined(i: int) {
    ///     o: int = 0 fby j;
    ///     out j: int = i + 1;
    /// }
    ///
    /// component calling_node(i: int) {
    ///     out o: int = to_be_inlined(o);
    ///     j: int = i * o;
    /// }
    /// ```
    ///
    /// The call to `to_be_inlined` will generate th following statements:
    ///
    /// ```GR
    /// o: int = 0 fby j_1;
    /// j_1: int = o + 1;
    /// ```
    pub fn instantiate_statements_and_memory(
        &self,
        identifier_creator: &mut IdentifierCreator,
        mut context_map: HashMap<usize, Either<usize, stream::Expr>>,
        symbol_table: &mut Ctx,
    ) -> (Vec<stream::Stmt>, Memory) {
        // add identifiers of the inlined statements to the context
        self.statements.iter().for_each(|statement| {
            statement.add_necessary_renaming(identifier_creator, &mut context_map, symbol_table)
        });
        // add identifiers of the inlined memory to the context
        self.memory
            .add_necessary_renaming(identifier_creator, &mut context_map, symbol_table);

        // reduce statements according to the context
        let statements = self
            .statements
            .iter()
            .map(|statement| statement.replace_by_context(&context_map))
            .collect();

        // reduce memory according to the context
        let memory = self.memory.replace_by_context(&context_map, &symbol_table);

        (statements, memory)
    }

    /// Update component statements and add the corresponding dependency graph.
    fn update_statements(&mut self, new_statements: &[stream::Stmt]) {
        // put new statements in component
        self.statements = new_statements.to_vec();
        // add a dependency graph to the component
        let mut graph = GraphMap::new();
        self.get_signals_id().iter().for_each(|signal_id| {
            graph.add_node(*signal_id);
        });
        self.statements
            .iter()
            .for_each(|statement| statement.add_to_graph(&mut graph));
        self.graph = graph;
    }

    /// Schedule statements.
    ///
    /// # Example.
    ///
    /// ```GR
    /// component test(v: int) {
    ///     out y: int = x-1
    ///     o_1: int = 0 fby x
    ///     x: int = v*2 + o_1
    /// }
    /// ```
    ///
    /// In the component above, signal `y` depends on the current value of `x`, `o_1` depends on the
    /// memory of `x` and `x` depends on `v` and `o_1`. The component is causal and should be scheduled
    /// as bellow:
    ///
    /// ```GR
    /// component test(v: int) {
    ///     o_1: int = 0 fby x  // depends on no current values of signals
    ///     x: int = v*2 + o_1  // depends on the computed value of `o_1` and given `v`
    ///     out y: int = x-1    // depends on the computed value of `x`
    /// }
    /// ```
    pub fn schedule(&mut self) {
        // get subgraph with only direct dependencies
        let mut subgraph = self.graph.clone();
        self.graph
            .all_edges()
            .for_each(|(from, to, label)| match label {
                Label::Weight(0) => (),
                _ => {
                    let res = subgraph.remove_edge(from, to);
                    debug_assert_ne!(res, Some(Label::Weight(0)))
                }
            });

        // topological sorting
        let mut schedule = toposort(&subgraph, None).unwrap();
        schedule.reverse();

        // construct map from signal_id to their position in the schedule
        let signals_order = schedule
            .into_iter()
            .enumerate()
            .map(|(order, signal_id)| (signal_id, order))
            .collect::<HashMap<_, _>>();
        let compare = |statement: &stream::Stmt| {
            statement
                .pattern
                .identifiers()
                .into_iter()
                .map(|signal_id| signals_order.get(&signal_id).unwrap())
                .min()
                .unwrap()
        };

        // sort statements
        self.statements.sort_by_key(compare);
        self.statements.iter_mut().for_each(|statement| {
            match &mut statement.expr.kind {
                stream::Kind::Expression { expr } => match expr {
                    ir1::expr::Kind::Match { arms, .. } => arms
                        .iter_mut()
                        .for_each(|(_, _, statements, _)| statements.sort_by_key(compare)),
                    _ => (),
                },
                _ => (),
            };
        })
    }
}

#[derive(Debug, Clone)]
/// LanGRust component import.
pub struct ComponentSignature {
    /// Component identifier.
    pub id: usize,
    /// Component location.
    pub loc: Loc,
    /// Component reduced dependency graph.
    pub reduced_graph: DiGraphMap<usize, Label>,
}

impl PartialEq for ComponentSignature {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.loc == other.loc && self.eq_graph(other)
    }
}

impl ComponentSignature {
    pub fn new(id: usize, loc: Loc) -> Self {
        Self {
            id,
            loc,
            reduced_graph: graph::DiGraphMap::new(),
        }
    }

    fn eq_graph(&self, other: &ComponentSignature) -> bool {
        let graph_nodes = self.reduced_graph.nodes();
        let other_nodes = other.reduced_graph.nodes();
        let graph_edges = self.reduced_graph.all_edges();
        let other_edges = other.reduced_graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }
}
