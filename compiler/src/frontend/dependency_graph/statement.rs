prelude! {
    graph::*,
    hir::{ Stmt, stream },
    frontend::ctx::*,
}

impl Stmt<stream::Expr> {
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
        let Stmt {
            expression,
            location,
            ..
        } = self;

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
                if expression.dependencies.get().is_none() {
                    expression.compute_dependencies(ctx)?;
                }

                // add dependencies as graph's edges:
                // s = e depends on s' <=> s -> s'
                expression
                    .get_dependencies()
                    .iter()
                    .for_each(|(id, label)| {
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
                    location: location.clone(),
                };
                ctx.errors.push(error);
                Err(TerminationError)
            }
            // if processed: nothing to do
            Color::Black => Ok(()),
        }
    }

    pub fn get_identifiers(&self) -> Vec<usize> {
        let mut identifiers = match &self.expression.kind {
            stream::Kind::Expression { expression } => match expression {
                hir::expr::Kind::Match { arms, .. } => arms
                    .iter()
                    .flat_map(|(pattern, _, statements, _)| {
                        statements
                            .iter()
                            .flat_map(|statement| statement.get_identifiers())
                            .chain(pattern.identifiers())
                    })
                    .collect(),
                _ => vec![],
            },
            _ => vec![],
        };

        identifiers.append(&mut self.pattern.identifiers());
        identifiers
    }
}
