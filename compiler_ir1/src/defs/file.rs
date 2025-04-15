//! [File] module.

prelude! {}

/// A LanGRust [File] is composed of functions, components,
/// types defined by the user, components and interface.
pub struct File {
    /// Program types.
    pub typedefs: Vec<ir1::Typedef>,
    /// Program functions.
    pub functions: Vec<ir1::Function>,
    /// Program components. They are functional requirements.
    pub components: Vec<ir1::Component>,
    /// Program interface. It represents the system.
    pub interface: ir1::Interface,
    /// Program location.
    pub loc: Loc,
}

impl File {
    /// Tell if it is in normal form.
    ///
    /// - component application as root expression
    /// - no rising edge
    pub fn is_normal_form(&self) -> bool {
        self.components
            .iter()
            .all(|component| component.is_normal_form())
    }

    /// Tell if there is no component application.
    pub fn no_component_application(&self) -> bool {
        self.components
            .iter()
            .all(|component| component.no_component_application())
    }

    /// Check the causality of the file.
    ///
    /// # Example
    ///
    /// The following file is causal, there is no causality loop.
    ///
    /// ```GR
    /// node causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the file that follows is not causal. In the node `not_causal_node`, signal`o` depends on
    /// `x` which depends on `o`. Values of signals can not be determined, then the compilation
    /// raises a causality error.
    ///
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    pub fn causality_analysis(&self, ctx: &Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        // check causality for each node
        self.components
            .iter()
            .map(|node| node.causal(ctx, errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<TRes<_>>()
    }

    /// Create memory for [ir1] nodes' unitary nodes.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
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
    ///
    /// This example is tested in source.
    pub fn memorize(&mut self, ctx: &mut Ctx) -> Res<()> {
        for comp in self.components.iter_mut() {
            comp.memorize(ctx)?;
        }
        for service in self.interface.services.iter_mut() {
            service.memorize(ctx)?;
        }
        Ok(())
    }

    /// Change [ir1] file into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    /// - no rising edges (replaced by |test| test && ! (false fby test))
    ///
    /// The normal form of a flow expression is as follows:
    /// - flow expressions others than identifiers are root expression
    /// - then, arguments are only identifiers
    ///
    /// # Example
    ///
    /// ```GR
    /// function test(i: int) -> int {
    ///     let x: int = i;
    ///     return x;
    /// }
    /// node my_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node other_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above node contains the following unitary nodes:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// node test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are transformed into:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// node test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    ///
    /// This example is tested in source.
    pub fn normal_form(&mut self, ctx: &mut Ctx) {
        let mut nodes_reduced_graphs = HashMap::new();
        // get every nodes' graphs
        self.components.iter().for_each(|node| {
            let _unique = nodes_reduced_graphs
                .insert(node.get_id().clone(), node.get_reduced_graph().clone());
            debug_assert!(_unique.is_none())
        });
        // normalize nodes
        self.components
            .iter_mut()
            .for_each(|node| node.normal_form(&nodes_reduced_graphs, ctx));

        // normalize interface
        self.interface.normal_form(ctx);

        // Debug: test it is in normal form
        debug_assert!(self.is_normal_form());
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// node fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed(&mut self, ctx: &mut Ctx) {
        let nodes = self
            .components
            .iter()
            .map(|node| (node.get_id().clone(), node.clone()))
            .collect::<HashMap<_, _>>();
        self.components
            .iter_mut()
            .for_each(|node| node.inline_when_needed(&nodes, ctx))
    }

    /// Schedule unitary nodes' equations.
    ///
    /// # Example.
    ///
    /// ```GR
    /// node test(v: int) {
    ///     out y: int = x-1
    ///     o_1: int = 0 fby x
    ///     x: int = v*2 + o_1
    /// }
    /// ```
    ///
    /// In the node above, signal `y` depends on the current value of `x`,
    /// `o_1` depends on the memory of `x` and `x` depends on `v` and `o_1`.
    /// The node is causal and should be scheduled as bellow:
    ///
    /// ```GR
    /// node test(v: int) {
    ///     o_1: int = 0 fby x  // depends on no current values of signals
    ///     x: int = v*2 + o_1  // depends on the computed value of `o_1` and given `v`
    ///     out y: int = x-1    // depends on the computed value of `x`
    /// }
    /// ```
    pub fn schedule(&mut self) {
        self.components.iter_mut().for_each(|node| node.schedule())
    }

    /// Generate dependency graphs for the interface.
    #[inline]
    pub fn generate_flows_dependency_graphs(&mut self) {
        self.interface.generate_flows_dependency_graphs()
    }

    /// Normalize [ir1] nodes in file.
    ///
    /// This is a chain of the following computations:
    /// - unitary nodes generation (check also that all signals are used)
    /// - inlining unitary node calls when needed (shifted causality loops)
    /// - scheduling unitary nodes
    /// - normalizing unitary node application
    /// - memorize node calls and followed by
    ///
    /// # Example
    ///
    /// Let be a node `my_node` and a node `other_node` as follows:
    ///
    /// ```GR
    /// node mem(i: int) {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int, g: int) {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(g-1, v-1).o2;
    ///     out z: int = mem(z).o;
    /// }
    /// ```
    ///
    /// ## Generate unitary nodes
    ///
    /// The generated unitary nodes are the following:
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(v-1).o2;
    /// }
    /// node other_node().z {
    ///     out z: int = mem(z).o;
    /// }
    /// ```
    ///
    /// But `g` is then unused, this will raise an error and stop the compilation.
    ///
    /// ## Inlining unitary nodes
    ///
    /// Suppose that we did not write `g` in the code and that the compilation
    /// succeeded the unitary node generation step. The inlining step will modify
    /// the unitary nodes as follows:
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(v-1).o2;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    ///
    /// In this example, `other_node` calls `mem` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `z` is defined before the input `z`,
    /// which can not be computed by a function call.
    ///
    /// ## Scheduling unitary nodes
    ///
    /// The scheduling step will order the equations of the unitary nodes.
    /// In our example, this will modify the code as bellow.
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     y: int = 1 + my_node(v-1).o2;         // y is before x now
    ///     out x: int = my_node(y, v).o1;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    ///
    /// ## Normal for of unitary nodes
    ///
    /// The last step is the final normal form of the unitary nodes.
    /// The normal form of an unitary node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// This correspond in our example to the following code:
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     x_1: int = v-1;             // x_1 was created
    ///     x_2: int = my_node(x_1).o2; // x_2 was created
    ///     y: int = 1 + x_2;
    ///     out x: int = my_node(y, v).o1;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    pub fn normalize(&mut self, ctx: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        self.normal_form(ctx);
        self.generate_flows_dependency_graphs();
        self.memorize(ctx).dewrap(errors)?;
        self.inline_when_needed(ctx);
        self.schedule();
        Ok(())
    }
}

pub mod dump_graph {
    prelude! {}
    use compiler_common::json::{begin_json, end_json};

    impl File {
        /// Dump dependency graph with parallelization weights.
        pub fn dump_graph<P: AsRef<std::path::Path>>(&self, filepath: P, ctx: &Ctx) {
            begin_json(&filepath);
            self.components
                .iter()
                .for_each(|comp| comp.dump_graph(&filepath, ctx));
            end_json(&filepath);
        }
    }
}
