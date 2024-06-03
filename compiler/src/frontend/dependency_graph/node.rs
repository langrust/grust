prelude! {
    graph::*,
    hir::Node,
    frontend::ctx::*,
}

impl Node {
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
            .filter(|id| !symbol_table.get_type(**id).is_event())
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

    /// Create an initialized processus manager from a node.
    fn create_initialized_processus_manager(
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

        // if component has events, add 'event' with white color (unprocessed)
        if let Some(event) = symbol_table.get_node_event(self.id) {
            hash.insert(event, Color::White);
        }

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
                .expression
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
        let mut processus_manager = self.create_initialized_processus_manager(ctx.symbol_table);

        // scope for inner `ctx`
        {
            let mut ctx = ctx.as_proc_ctx(&mut processus_manager);
            // add local and output signals dependencies
            for s in self.statements.iter() {
                s.add_dependencies(&mut ctx)?
            }
        }

        // add input signals dependencies
        ctx.symbol_table
            .get_node_inputs(self.id)
            .iter()
            .filter(|id| !ctx.symbol_table.get_type(**id).is_event())
            .for_each(|signal| {
                // get signal's color
                let color = processus_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;
            });

        Ok(())
    }

    fn construct_reduced_graph(&self, ctx: &mut Ctx) {
        ctx.reduced_graphs
            .insert(self.id, self.create_initialized_graph(ctx.symbol_table));

        let mut processus_manager = self.create_initialized_processus_manager(ctx.symbol_table);

        // add output dependencies over inputs in reduced graph
        ctx.symbol_table
            .get_node_outputs(self.id)
            .iter()
            .for_each(|(_, output_signal)| {
                self.add_signal_dependencies_over_inputs(
                    *output_signal,
                    ctx,
                    &mut processus_manager,
                )
            });
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
        processus_manager: &mut HashMap<usize, Color>,
    ) {
        let Node { id: node, .. } = self;

        // get signal's color
        let color = processus_manager.get_mut(&signal).expect(&format!(
            "signal '{}' should be in processus manager",
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
                        self.add_signal_dependencies_over_inputs(
                            neighbor_id,
                            ctx,
                            processus_manager,
                        );

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
                let color = processus_manager
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
