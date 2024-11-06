prelude! {
    graph::*,
    hir::{expr, stream},
}

use super::ctx;

impl stream::ExprKind {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self) -> Vec<usize> {
        match &self {
            Self::Constant { .. } | Self::Identifier { .. } | Self::Enumeration { .. } => vec![],
            Self::Application {
                function_expression,
                inputs,
            } => {
                let mut nodes = inputs
                    .iter()
                    .flat_map(|expression| expression.get_called_nodes())
                    .collect::<Vec<_>>();
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Abstraction { expression, .. } | Self::Unop { expression, .. } => {
                expression.get_called_nodes()
            }
            Self::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                let mut nodes = left_expression.get_called_nodes();
                let mut other_nodes = right_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = true_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                let mut other_nodes = false_expression.get_called_nodes();
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
            Self::Match { expression, arms } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = arms
                    .iter()
                    .flat_map(|(_, bound, body, expression)| {
                        let mut nodes = vec![];
                        body.iter().for_each(|statement| {
                            let mut other_nodes = statement.expression.get_called_nodes();
                            nodes.append(&mut other_nodes);
                        });
                        let mut other_nodes = expression.get_called_nodes();
                        nodes.append(&mut other_nodes);
                        let mut other_nodes = bound
                            .as_ref()
                            .map_or(vec![], |expression| expression.get_called_nodes());
                        nodes.append(&mut other_nodes);
                        nodes
                    })
                    .collect::<Vec<_>>();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::FieldAccess { expression, .. } => expression.get_called_nodes(),
            Self::TupleElementAccess { expression, .. } => expression.get_called_nodes(),
            Self::Map {
                expression,
                function_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = initialization_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Sort {
                expression,
                function_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            Self::Zip { arrays } => arrays
                .iter()
                .flat_map(|expression| expression.get_called_nodes())
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
    pub fn compute_dependencies(&self, ctx: &mut ctx::GraphProcCtx) -> TRes<Vec<(usize, Label)>> {
        use expr::Kind::*;
        match self {
            Constant { .. } => Self::constant_deps(),
            Identifier { id, .. } => Self::ident_deps(ctx.symbol_table, *id),
            Abstraction { .. } => Self::abstraction_deps(),
            Enumeration { .. } => Self::enumeration_deps(),
            Unop { expression, .. } => Self::unop_deps(ctx, expression),
            Binop {
                left_expression,
                right_expression,
                ..
            } => Self::binop_deps(ctx, left_expression, right_expression),
            IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => Self::ite_deps(ctx, expression, true_expression, false_expression),
            Application {
                function_expression,
                inputs,
                ..
            } => Self::fun_app_deps(ctx, function_expression, inputs),
            Structure { fields, .. } => Self::structure_deps(ctx, fields),
            Array { elements } => Self::array_deps(ctx, elements),
            Tuple { elements } => Self::tuple_deps(ctx, elements),
            Match { expression, arms } => Self::match_deps(&self, ctx, expression, arms),
            FieldAccess { expression, .. } => Self::field_access_deps(&self, ctx, expression),
            TupleElementAccess { expression, .. } => Self::tuple_access_deps(ctx, expression),
            Map { expression, .. } => Self::map_deps(ctx, expression),
            Fold {
                expression,
                initialization_expression,
                ..
            } => Self::fold_deps(ctx, expression, initialization_expression),
            Sort { expression, .. } => Self::sort_deps(&self, ctx, expression),
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
        ctx: &mut ctx::GraphProcCtx,
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
        ctx: &mut ctx::GraphProcCtx,
        elems: &Vec<stream::Expr>,
    ) -> TRes<Vec<(usize, Label)>> {
        let mut res = Vec::with_capacity(elems.len());
        // propagate dependencies computation
        for e in elems.iter() {
            e.compute_dependencies(ctx)?;
            res.extend(e.get_dependencies().iter().cloned());
        }
        Ok(res)
    }

    /// Compute dependencies of a binop stream expression.
    fn binop_deps(
        ctx: &mut ctx::GraphProcCtx,
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
        ctx: &mut ctx::GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get accessed expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a fold stream expression.
    fn fold_deps(
        ctx: &mut ctx::GraphProcCtx,
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

    /// Compute dependencies of a ifthenelse stream expression.
    pub fn ite_deps(
        ctx: &mut ctx::GraphProcCtx,
        c: &stream::Expr,
        t: &stream::Expr,
        e: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // dependencies of ifthenelse are dependencies of the expressions
        c.compute_dependencies(ctx)?;
        t.compute_dependencies(ctx)?;
        e.compute_dependencies(ctx)?;

        let mut deps = c.get_dependencies().clone();
        deps.extend(t.get_dependencies().iter().cloned());
        deps.extend(e.get_dependencies().iter().cloned());

        Ok(deps)
    }

    /// Compute dependencies of a map stream expression.
    pub fn map_deps(ctx: &mut ctx::GraphProcCtx, expr: &stream::Expr) -> TRes<Vec<(usize, Label)>> {
        // get mapped expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a match stream expression.
    pub fn match_deps(
        &self,
        ctx: &mut ctx::GraphProcCtx,
        expr: &stream::Expr,
        arms: &Vec<(
            hir::pattern::Pattern,
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
                add_deps!(statement.expression.get_dependencies().iter());
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
        ctx: &mut ctx::GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get sorted expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a structure stream expression.
    pub fn structure_deps(
        ctx: &mut ctx::GraphProcCtx,
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
        ctx: &mut ctx::GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get accessed expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of an tuple stream expression.
    pub fn tuple_deps(
        ctx: &mut ctx::GraphProcCtx,
        elems: &Vec<stream::Expr>,
    ) -> TRes<Vec<(usize, Label)>> {
        let mut deps = Vec::with_capacity(25);
        // propagate dependencies computation
        for e in elems.iter() {
            e.compute_dependencies(ctx)?;
            deps.extend(e.get_dependencies().iter().cloned())
        }
        // not shrinking, this might grow later
        Ok(deps)
    }

    /// Compute dependencies of a unop stream expression.
    pub fn unop_deps(
        ctx: &mut ctx::GraphProcCtx,
        expr: &stream::Expr,
    ) -> TRes<Vec<(usize, Label)>> {
        // get expression dependencies
        expr.compute_dependencies(ctx)?;
        Ok(expr.get_dependencies().clone())
    }

    /// Compute dependencies of a zip stream expression.
    pub fn zip_deps(
        ctx: &mut ctx::GraphProcCtx,
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
