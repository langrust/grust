//! # Parallel compilation of statements
//!
//! This module is heavily documented and should be relatively easy to understand.
//!
//! The exception is *fast-rayon* code generation. This parallelization approach is a bit strange
//! but works well a big number of very small computations.
//!
//! ## Fast-rayon
//!
//! Say you want to run `n` computations `c_1, ..., c_n` producing results of potentially different
//! types, and say we want to use rayon to parallelize `let (id_1, ..., id_n) = (c_1, ..., c_n);`.
//!
//! Rayon works by building a parallel iterator, and then reducing (or folding) over it to actually
//! run computations concurrently and *aggregate* a result. Our problem is that we just want to
//! retrieve the result of each computation separately. We can't do that by side-effect because of
//! how rayon's design and rust's borrow checker blend together.
//!
//! What we can do is *reduce* to the `n`-tuple where the element at `i` is the result of `c_i`. We
//! can't do this directly because each element of the tuple is built separately, **and inserted
//! into the final tuple** separately. Basically, elements of the tuple are going to be options (all
//! `None` at the start of the *reduce*) and the *reduce* will set relevant elements to `Some
//! <res_i>` as it goes.
//!
//! For example:
//!
//! ```rust
//! // here are three computations or whatever
//! fn c_1(n: usize) -> usize {
//!     5 + n
//! }
//! fn c_2(s: &str) -> String {
//!     let mut s = s.to_string();
//!     s.push_str(" adding something");
//!     s
//! }
//! fn c_3(f: f64) -> f64 {
//!     f * 3.14
//! }
//!
//! // sequential version
//! let (res_1, res_2, res_3) = (c_1(2), c_2("some text"), c_3(4.2));
//! # println!("res_1: {res_1}, res_2: {res_2}, res_3: {res_3}");
//! assert_eq!(res_1, 7);
//! assert_eq!(&res_2, "some text adding something");
//! assert_eq!(res_3, 13.188);
//!
//! use rayon::prelude::*;
//!
//! // rayon-fast version
//! let (rres_1, rres_2, rres_3) = {
//!     let (r1, r2, r3) = (1..=3)
//!         .into_par_iter()
//!         .map(|idx| match idx {
//!             1 => (Some(c_1(2)), None, None),
//!             2 => (None, Some(c_2("some text")), None),
//!             3 => (None, None, Some(c_3(4.2))),
//!             _ => unreachable!(),
//!         })
//!         .reduce(
//!             || (None, None, None),
//!             |lft, rgt| {
//!                 let r1 = match (lft.0, rgt.0) {
//!                     (None, res) | (res, None) => res,
//!                     (Some(_), Some(_)) => unreachable!(),
//!                 };
//!                 let r2 = match (lft.1, rgt.1) {
//!                     (None, res) | (res, None) => res,
//!                     (Some(_), Some(_)) => unreachable!(),
//!                 };
//!                 let r3 = match (lft.2, rgt.2) {
//!                     (None, res) | (res, None) => res,
//!                     (Some(_), Some(_)) => unreachable!(),
//!                 };
//!                 (r1, r2, r3)
//!             },
//!         );
//!     (r1.unwrap(), r2.unwrap(), r3.unwrap())
//! };
//! # println!("rres_1: {rres_1}, rres_2: {rres_2}, rres_3: {rres_3}");
//!
//! assert_eq!(res_1, rres_1);
//! assert_eq!(res_2, rres_2);
//! assert_eq!(res_3, rres_3);
//! ```

prelude! {
    BTreeMap as Map,
    // BTreeSet as Set,
    synced::{generic::Synced, Weight, weight},
}

#[allow(unused_macros)]
macro_rules! token_show {
    { $vec:expr, $($stuff:tt)* } => {
        println!($($stuff)*);
        for elm in $vec {
            println!("- {}", elm.to_token_stream());
        }
    };
}

/// Enumeration of the different kinds of parallelization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParaKind {
    /// Naive rayon with options, fast on many small computations.
    RayonFast,
    /// Normal rayon, fast on relatively small computations.
    Rayon,
    /// System threads, heavyweight.
    Threads,
    /// Tokio threads, heavyweight.
    TokioThreads,
    /// No parallelization, *i.e.* sequential statements.
    None,
}
impl Display for ParaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "none".fmt(f),
            Self::RayonFast => "rayon_fast".fmt(f),
            Self::Rayon => "rayon".fmt(f),
            Self::Threads => "threads".fmt(f),
            Self::TokioThreads => "tokio_threads".fmt(f),
        }
    }
}

// fn use_rayon_prelude() -> syn::ItemUse {
//     parse_quote! { use grust::grust_std::rayon::prelude::*; }
// }
// fn rayon_join() -> syn::Path {
//     parse_quote! { grust::grust_std::rayon::join }
// }

/// Type alias for a directed graph with statement UIDs (`usize`) nodes and graph label vertices.
pub type Graph = graph::DiGraphMap<usize, graph::Label>;

/// Parallelization environment, stores the graph and data about statements.
pub struct Env<'a> {
    /// Map a identifier UID to its representative identifier UID.
    ///
    /// Statements can bind more than one identifier at a time, in tuple-deconstruction for
    /// instance. Still, all these identifiers are defined at the same time so from the point of
    /// view of scheduling parallel statements we can just pick one of these identifiers as the
    /// *representative* `id_k` of the *class* of identifiers `{id_1, ..., id_n}` bound by a
    /// statement `stmt`; `id_to_repr` maps `id_i` to `id_k` for all statements.
    ///
    /// That way, we can construct `graph` (another field in this structure) that simply use
    /// representatives as nodes.
    id_to_repr: Map<usize, usize>,
    /// Maps representative UIDs to the statement binding it (and all of the identifiers in its
    /// class).
    repr_to_stmt: Map<usize, &'a ir1::stream::Stmt>,
    /// Dependency graph between representative UIDs (labels/graph weights are irrelevant).
    graph: Graph,
    weight_bounds: synced::WeightBounds,
}
impl<'a> Env<'a> {
    /// Applies `f` to all identifiers appearing in pattern `kind`.
    fn ids_do(kind: &ir1::stmt::Kind, f: &mut impl FnMut(usize)) -> Result<(), String> {
        use ir1::stmt::Kind::*;
        let mut curr = kind;
        let mut stack = vec![];
        'current: loop {
            match curr {
                Identifier { id } | Typed { id, .. } => f(*id),
                Tuple { elements } => {
                    let mut elms = elements.iter().map(|pat| &pat.kind);
                    let first = elms.next().ok_or_else(|| "empty tuple in let-binding")?;
                    curr = &first;
                    stack.extend(elms);
                    continue 'current;
                }
            }
            while let Some(next) = stack.pop() {
                curr = next;
                continue 'current;
            }
            return Ok(());
        }
    }

    /// Edge-transitive-closure (`edge_tc`) `graph`.
    ///
    /// The graph `g` returned is the transitive closure of `graph`: for all nodes `a b c ∈ graph`,
    /// if `a → b → c ∈ graph` then `a → c ∈ g`.
    fn graph_edge_tc(graph: &Graph) -> Graph {
        let mut res = Graph::new();
        let mut known = BTreeSet::new();
        let mut todo = Vec::with_capacity(graph.node_count());
        for src in graph.nodes() {
            known.clear();
            debug_assert!(todo.is_empty());

            todo.push(src);
            let is_new = known.insert(src);
            debug_assert!(is_new);

            while let Some(node) = todo.pop() {
                for (_, tgt, label) in graph.edges_directed(node, graph::Direction::Outgoing) {
                    let is_new = known.insert(tgt);
                    if label.has_weight(0) && is_new {
                        res.add_edge(src, tgt, label.clone());
                        todo.push(tgt);
                    }
                }
            }
        }
        res
    }

    /// Retrieves the representative of an identifier UID.
    pub fn repr_of(&self, id: usize) -> Option<usize> {
        self.id_to_repr.get(&id).cloned()
    }

    /// Constructor.
    ///
    /// Builds the map between identifier UIDs and their representative (discussed in [Env]), the
    /// map between representatives and the corresponding statement, and the graph of dependency
    /// between representatives.
    pub fn new(
        stmts: &'a Vec<ir1::stream::Stmt>,
        graph: &Graph,
        weight_bounds: synced::WeightBounds,
    ) -> Result<Self, String> {
        let graph = Self::graph_edge_tc(graph);
        let mut slf = Self {
            id_to_repr: Map::new(),
            repr_to_stmt: Map::new(),
            graph: Graph::new(),
            weight_bounds,
        };
        for stmt in stmts {
            let mut repr = None;
            Self::ids_do(&stmt.pattern.kind, &mut |id| {
                if let Some(repr) = repr {
                    let prev = slf.id_to_repr.insert(id, repr);
                    debug_assert!(prev.is_none());
                } else {
                    let prev = slf.repr_to_stmt.insert(id, stmt);
                    debug_assert!(prev.is_none());
                    let prev = slf.id_to_repr.insert(id, id);
                    debug_assert!(prev.is_none());
                    repr = Some(id);
                }
            })?
        }
        for (src, tgt, label) in graph.all_edges() {
            use graph::Label;
            match label {
                Label::Weight(0) => (),
                Label::Weight(_) | Label::Contract => continue,
            }
            match (slf.repr_of(src), slf.repr_of(tgt)) {
                (None, None) => (),
                (Some(src), Some(tgt)) => {
                    let _ = slf.graph.add_edge(src, tgt, *label);
                }
                (Some(src), None) => {
                    let _ = slf.graph.add_edge(src, tgt, *label);
                }
                (None, Some(tgt)) => {
                    let _ = slf.graph.add_edge(src, tgt, *label);
                }
            }
        }
        Ok(slf)
    }

    /// Generates the parallel code for the statements.
    pub fn to_stmts(&self, ctx: &ir0::Ctx) -> Result<Stmts, String> {
        let subset = self.repr_to_stmt.keys().cloned().collect();
        let synced = Synced::new_with(self, &self.graph, subset)?;
        Stmts::of_synced(self, ctx, synced)
    }

    /// String representation, used for debugging.
    pub fn print(&self, ctx: &ir0::Ctx) {
        println!("env:");
        println!("  idents:");
        for tgt in self.id_to_repr.values() {
            let name = ctx.get_name(*tgt);
            println!("    {} is {}", tgt, name);
            for (_, tgt, label) in self.graph.edges_directed(*tgt, graph::Direction::Outgoing) {
                println!("      dep {} ({}, {:?})", ctx.get_name(tgt), tgt, label);
                for (_, tgt, label) in self.graph.edges_directed(tgt, graph::Direction::Outgoing) {
                    println!("        dep {} ({}, {:?})", ctx.get_name(tgt), tgt, label);
                }
            }
        }
        println!("  id_to_repr:");
        for (src, tgt) in &self.id_to_repr {
            println!("    {} ↦ {}", src, tgt);
        }
    }

    /// True on edge labels that should be ignored, used in the [synced::CtxSpec] implementation.
    pub fn ignore_edge(label: &graph::Label) -> bool {
        !label.has_weight(0)
    }

    /// Computes the cost of an instruction, used in the [synced::CtxSpec] implementation.
    pub fn instr_cost(&self, uid: Weight) -> Weight {
        let stmt = self
            .repr_to_stmt
            .get(&uid)
            .expect("unknown instruction uid");
        stmt.weight(&self.weight_bounds)
    }

    /// Cost of a sequence of statements, used in the implementation of [synced::CtxSpec].
    pub fn sync_seq_cost(&self, seq: &[Synced<Self>]) -> Weight {
        weight::from_usize(seq.len()) + w8!(sum seq, |seq| seq.cost())
    }

    /// Cost of some parallel statements, used in the implementation of [synced::CtxSpec].
    pub fn sync_para_cost(&self, map: &BTreeMap<Weight, Vec<Synced<Self>>>) -> Weight {
        let (max, count) = map.iter().fold(
            (weight::zero, weight::zero),
            |(max, count), (key, branches)| (max.max(*key), count + branches.len()),
        );
        weight::from_usize(max + count)
    }
}

impl<'a> synced::generic::CtxSpec for Env<'a> {
    type Instr = usize;
    type Cost = Weight;
    type Label = graph::Label;
    const INVERTED_EDGES: bool = true;
    fn ignore_edge(label: &Self::Label) -> bool {
        Env::ignore_edge(label)
    }
    fn instr_cost(&self, uid: usize) -> usize {
        self.instr_cost(uid)
    }
    fn sync_seq_cost(&self, seq: &[Synced<Self>]) -> usize {
        self.sync_seq_cost(seq)
    }
    fn sync_para_cost(&self, map: &BTreeMap<usize, Vec<Synced<Self>>>) -> usize {
        self.sync_para_cost(map)
    }
}

/// Type alias [ir1] patterns.
pub type Bindings = ir1::stmt::Pattern;

/// A variable plain/(nested) tuple pattern.
///
/// Stores the [Pattern] and the [Expr] version of a (tuple of) variable(s). This is useful during
/// code generation, as we need to juggle between tuples of variables as patterns and as
/// expressions. Especially when we do parallel code generation and we need to handle variable
/// plumbing.
#[derive(Debug, Clone, PartialEq)]
pub struct Vars {
    /// [Pattern] version of the variables.
    bind: Pattern,
    /// [Expr] version of the variables.
    expr: Expr,
}
impl Vars {
    /// Binding (pattern) accessor.
    pub fn as_pattern(&self) -> &Pattern {
        &self.bind
    }
    /// Expression accessor.
    pub fn as_expr(&self) -> &Expr {
        &self.expr
    }

    /// Merges some [Vars] structure as a (nested) tuple.
    pub fn tuple_merge(vars: impl IntoIterator<Item = Self> + ExactSizeIterator) -> Self {
        let (mut bind_vec, mut expr_vec) = (
            Vec::with_capacity(vars.len()),
            Vec::with_capacity(vars.len()),
        );
        for var in vars {
            bind_vec.push(var.bind);
            expr_vec.push(var.expr);
        }
        debug_assert_eq!(bind_vec.len(), expr_vec.len());
        let (bind, expr) = if bind_vec.len() == 1 {
            (bind_vec.pop().unwrap(), expr_vec.pop().unwrap())
        } else {
            (Pattern::tuple(bind_vec), Expr::tuple(expr_vec))
        };
        Self { bind, expr }
    }

    /// Creates the binding `let <id> = <id>:`.
    pub fn new(id: Ident) -> Self {
        Self {
            bind: Pattern::ident(id.clone()),
            expr: Expr::ident(id),
        }
    }

    /// Creates the binding `let <pat> = <pat>;` where `pat` is an [ir1] pattern.
    pub fn of_ir1(pat: ir1::stmt::Pattern, ctx: &ir0::Ctx) -> Self {
        let mut curr = pat.kind;
        let mut stack = vec![];

        'current: loop {
            use ir1::stmt::Kind;
            let (mut bind, mut expr) = match curr {
                Kind::Identifier { id } => {
                    let id = ctx.get_name(id).clone();
                    (Pattern::ident(id.clone()), Expr::ident(id))
                }
                Kind::Tuple { elements } => {
                    let mut elms = elements.into_iter().map(|pat| pat.kind);
                    curr = elms.next().expect("unexpected empty tuple pattern");
                    stack.push((elms, vec![], vec![]));
                    continue 'current;
                }
                Kind::Typed { id, typ } => {
                    let id = ctx.get_name(id).clone();
                    (
                        Pattern::typed(Pattern::ident(id.clone()), typ),
                        Expr::ident(id),
                    )
                }
            };

            'unstack: while let Some((mut iter, mut bind_vec, mut expr_vec)) = stack.pop() {
                debug_assert_eq!(bind_vec.len(), expr_vec.len());
                bind_vec.push(bind);
                expr_vec.push(expr);
                if let Some(next) = iter.next() {
                    curr = next;
                    stack.push((iter, bind_vec, expr_vec));
                    continue 'current;
                } else {
                    if bind_vec.len() == 1 {
                        bind = bind_vec.pop().unwrap();
                        expr = expr_vec.pop().unwrap();
                    } else {
                        bind = Pattern::tuple(bind_vec);
                        expr = Expr::tuple(expr_vec);
                    }
                    continue 'unstack;
                }
            }

            return Self { bind, expr };
        }
    }
}

/// A four-step parallel-code-execution scheduler.
///
/// Each of the four steps is a field of this type storing a sequence of statements.
///
/// This comes from the different kinds of parallelization we're doing. Some of it is not blocking,
/// such as (system or tokio) threads. We can (and do) *spawn* them, do stuff, and then *join* the
/// threads to get back the results.
///
/// Some of it is blocking, such as rayon-parallelized code. To use rayon we build a parallel
/// iterator and then `fold`/`reduce`/... over it, which runs the branches concurrently but blocks
/// us until all have returned and the result has been produced. We cannot do anything new while
/// this runs.
///
/// So, to have everything run in parallel, we need to make sure we
/// - [Self::spawn] the threads first;
/// - then [Self::run] blocking code as the threads are working until all blocking code is done and
///   we have its results;
/// - then [Self::join] the threads to retrieve their results;
/// - finally package all [Self::res]ults in a tuple/nested tuples for whatever is above us to use.
pub struct Schedule {
    /// Spawn step.
    ///
    /// Spawns threads if any, work in the threads start concurrently during the next step (`run`).
    pub spawn: Option<Vec<syn::Stmt>>,
    /// Run step.
    ///
    /// Run parallel-but-blocking code, typically rayon, threads (if any) still running from the
    /// previous step (`spawn`). Note that at the end of this step, we have all results for this
    /// blocking code: since it is blocking, it produces the results once all sub-tasks are done.
    pub run: Option<Vec<syn::Stmt>>,
    /// Join step.
    ///
    /// Wait on all the threads, retrieve results. Note that blocking code has already run in the
    /// previous step (`run`) and finished, thus we already have the results for that.
    pub join: Option<Vec<syn::Stmt>>,
    /// Result step.
    ///
    /// Aggregate all results in an appropriate tuple for whatever is above us.
    pub res: Vec<syn::Expr>,
}
mk_new! { impl Schedule => new {
    spawn: Option<Vec<syn::Stmt>>,
    run: Option<Vec<syn::Stmt>>,
    join: Option<Vec<syn::Stmt>>,
    res: Vec<syn::Expr>,
} }
impl Schedule {
    /// Empty scheduler, schedules nothing.
    pub fn empty() -> Schedule {
        Self::new(None, None, None, vec![])
    }
    /// Creates a pure-thread scheduler with no (empty) `run` step.
    pub fn threads(spawn: Vec<syn::Stmt>, join: Vec<syn::Stmt>, res: syn::Expr) -> Schedule {
        Self::new(Some(spawn), None, Some(join), vec![res])
    }
    /// Creates a sequential scheduler.
    ///
    /// This is useful to deploy code parallelized using (different versions of) rayon.
    pub fn sequence(seq: Vec<syn::Stmt>, res: syn::Expr) -> Schedule {
        Self::new(None, Some(seq), None, vec![res])
    }

    /// Produces a unique index for a new *spawn* statement.
    pub fn next_spawn_index(&self) -> usize {
        self.spawn.as_ref().map(|vec| vec.len()).unwrap_or(0)
    }

    /// Identifier used
    fn scope_ident() -> syn::Ident {
        syn::Ident::new("reserved_grust_thread_scope", Span::call_site())
    }

    /// Merges two optional vectors.
    ///
    /// The result is in `lft` and contains
    /// - `None` if `lft` and `rgt` are both `None`;
    /// - the concatenation of `lft.unwrap_or(vec![])` and `rgt.unwrap_or(vec![])` otherwise.
    fn merge_opt_vec<T>(lft: &mut Option<Vec<T>>, rgt: Option<Vec<T>>) {
        match (lft.as_mut(), rgt) {
            (_, None) => (),
            (None, Some(vec)) => *lft = Some(vec),
            (Some(lft), Some(rgt)) => lft.extend(rgt),
        }
    }
    /// Merges two schedulers by merging/extending all `self`-steps with the `that`-steps.
    pub fn merge(&mut self, that: Self) {
        Self::merge_opt_vec(&mut self.spawn, that.spawn);
        Self::merge_opt_vec(&mut self.run, that.run);
        Self::merge_opt_vec(&mut self.join, that.join);
        self.res.extend(that.res);
    }

    /// Turns itself into rust code.
    pub fn into_syn(self, dont_bind: bool) -> syn::Stmt {
        debug_assert!(
            self.spawn.is_none() && self.join.is_none()
                || self.spawn.is_some() && self.join.is_some()
        );
        assert!(!self.res.is_empty());
        let Self {
            spawn,
            run,
            join,
            mut res,
        } = self;
        let spawn = spawn.and_then(|vec| if vec.is_empty() { None } else { Some(vec) });
        // token_show!(
        //     spawn.as_ref().into_iter().map(|v| v.into_iter()).flatten(),
        //     "spawn:"
        // );
        // token_show!(
        //     run.as_ref().into_iter().map(|v| v.into_iter()).flatten(),
        //     "run:"
        // );
        // token_show!(
        //     join.as_ref().into_iter().map(|v| v.into_iter()).flatten(),
        //     "join:"
        // );
        let run = run.unwrap_or_else(Vec::new);
        let join = join.unwrap_or_else(Vec::new);
        let res = if res.len() > 1 {
            parse_quote! { ( #(#res),* ) }
        } else {
            res.pop().unwrap()
        };
        let body: syn::Expr = if let Some(spawn) = spawn {
            let scope = Self::scope_ident();
            parse_quote! {
                std::thread::scope(|#scope| {
                    #(#spawn)*
                    #(#run)*
                    #(#join)*
                    #res
                })
            }
        } else {
            parse_quote! {
                {
                    #(#run)*
                    #res
                }
            }
        };
        // println!("    body: {}", body.to_token_stream());
        if dont_bind {
            syn::Stmt::Expr(body, None)
        } else {
            parse_quote! {
                let #res = { #body };
            }
        }
    }
}

/// Part of the `ir2` AST representing (parallel) statements.
///
/// This inductive type does not actually mention statements *per se*, it really deals [Vars]/[Expr]
/// because all statements bind some identifiers. All constructors have a [Vars] value, which
/// are the ((nested) tuple of) variables that the statement(s) bind.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmts {
    /// A sequence of binding statements binding some [Vars].
    Seq(Vars, Vec<Self>),
    /// Some parallel statements binding some [Vars].
    Para(Vars, Vec<(ParaKind, Vars, Vec<Self>)>),
    /// A plain statement binding some [Vars] using an expression.
    Stmt(Vars, Expr),
}

mk_new! { impl Stmts =>
    Seq: seq (
        vars: Vars = vars,
        stmts: Vec<Self> = stmts,
    )
    Para: para(
        vars: Vars = vars,
        para: Vec<(ParaKind, Vars, Vec<Self>)> = para,
    )
    Stmt: stmt(
        vars: Vars = vars,
        expr: Expr = expr,
    )
}

impl Stmts {
    /// Statement sequence constructor.
    pub fn new_seq(stmts: Vec<Self>) -> Self {
        let vars = Vars::tuple_merge(stmts.iter().map(|stmt| stmt.vars().clone()));
        Self::Seq(vars, stmts)
    }
    /// Parallel statements constructor.
    pub fn new_para(subs: Vec<(ParaKind, Vars, Vec<Self>)>) -> Self {
        let vars = Vars::tuple_merge(subs.iter().map(|(_, vars, _)| vars.clone()));
        Self::Para(vars, subs)
    }
    /// Plain statement constructor.
    pub fn new_stmt(stmt: &ir1::stream::Stmt, ctx: &ir0::Ctx) -> Self {
        let vars = Vars::of_ir1(stmt.pattern.clone(), ctx);
        let expr = stmt.expr.clone().into_ir2(ctx);
        Self::Stmt(vars, expr)
    }

    /// Builds a sequence of statements from some [Ident]/[Expr] pairs.
    pub fn seq_of_pairs(bindings: impl IntoIterator<Item = (Ident, Expr)>) -> Self {
        Self::new_seq(
            bindings
                .into_iter()
                .map(|(id, expr)| Stmts::stmt(Vars::new(id), expr))
                .collect(),
        )
    }

    /// Builds a sequence of statements from some [ir1] statements.
    pub fn seq_of_ir1(stmts: &Vec<ir1::stream::Stmt>, ctx: &ir0::Ctx) -> Self {
        Self::new_seq(stmts.iter().map(|stmt| Self::new_stmt(stmt, ctx)).collect())
    }

    /// Constructor from [ir1] statements.
    pub fn of_ir1(
        stmts: &Vec<ir1::stream::Stmt>,
        ctx: &ir0::Ctx,
        graph: &Graph,
    ) -> Result<Self, String> {
        if let conf::ComponentPara::Para(weight_bounds) = ctx.conf.component_para {
            let env = Env::new(stmts, graph, weight_bounds)?;
            // env.print(ctx);
            env.to_stmts(ctx)
        } else {
            Ok(Self::seq_of_ir1(stmts, ctx))
        }
    }

    /// The variables bound by these statements.
    pub fn vars(&self) -> &Vars {
        match self {
            Self::Seq(vars, _) => vars,
            Self::Para(vars, _) => vars,
            Self::Stmt(vars, _) => vars,
        }
    }

    /// Constructor from an [Env] and some [Synced] data.
    pub fn of_synced(env: &Env, ctx: &ir0::Ctx, synced: Synced<Env>) -> Result<Self, String> {
        match synced {
            Synced::Instr(id, _) => {
                let stmt = env
                    .repr_to_stmt
                    .get(&id)
                    .ok_or_else(|| format!("unknown statement identifier `{}`", id))?;
                Ok(Self::new_stmt(stmt, ctx))
            }
            Synced::Seq(subs, _) => {
                let mut seq = Vec::with_capacity(subs.len());
                for sub in subs {
                    let sub = Self::of_synced(env, ctx, sub)?;
                    seq.push(sub)
                }
                Ok(Self::new_seq(seq))
            }
            Synced::Para(map, _) => {
                // println!("of_synced: para");
                let mut no_para = Vec::with_capacity(map.len());
                let mut no_para_vars = Vec::with_capacity(map.len());
                let mut rayon = Vec::with_capacity(map.len());
                let mut rayon_vars = Vec::with_capacity(map.len());
                let mut threads = Vec::with_capacity(map.len());
                let mut threads_vars = Vec::with_capacity(map.len());
                for (weight, subs) in map {
                    let para_mode = ctx.conf.component_para;
                    // println!(
                    //     "para mode: {:?}, weight is {} ({})",
                    //     para_mode,
                    //     weight,
                    //     para_mode.is_rayon(weight < Ctx::RAYON_PARA_WEIGHT_UBX)
                    // );
                    use synced::Kind::*;
                    let (target, target_vars) = {
                        // println!("deciding for weight `{}`", weight);
                        match para_mode.decide(weight) {
                            Seq => {
                                // println!("- Seq");
                                (&mut no_para, &mut no_para_vars)
                            }
                            FastRayon | Rayon => {
                                // println!("- FastRayon");
                                (&mut rayon, &mut rayon_vars)
                            }
                            Threads => {
                                // println!("- Threads");
                                (&mut threads, &mut threads_vars)
                            }
                        }
                    };
                    for sub in subs {
                        let sub = Self::of_synced(env, ctx, sub)?;
                        target_vars.push(sub.vars().clone());
                        target.push(sub);
                    }
                }
                // println!("ping");
                let mut paras = Vec::with_capacity(3);
                if !no_para.is_empty() {
                    // println!("pushing {} no-para(s)", no_para.len());
                    paras.push((
                        ParaKind::None,
                        Vars::tuple_merge(no_para_vars.into_iter()),
                        no_para,
                    ));
                }

                let rayon_is_empty = rayon.is_empty();
                if !rayon_is_empty {
                    if rayon.len() == 1 && threads.is_empty() {
                        paras.push((
                            ParaKind::None,
                            Vars::tuple_merge(rayon_vars.into_iter()),
                            rayon,
                        ));
                    } else {
                        // println!("pushing {} rayon(s)", rayon.len());
                        paras.push((
                            ParaKind::RayonFast,
                            Vars::tuple_merge(rayon_vars.into_iter()),
                            rayon,
                        ));
                    }
                }

                if threads.len() == 1 && rayon_is_empty {
                    paras.push((
                        ParaKind::None,
                        Vars::tuple_merge(threads_vars.into_iter()),
                        threads,
                    ));
                } else {
                    // println!("pushing {} thread(s)", threads.len());
                    paras.push((
                        ParaKind::Threads,
                        Vars::tuple_merge(threads_vars.into_iter()),
                        threads,
                    ));
                }
                Ok(Self::new_para(paras))
            }
        }
    }

    /// Compiles a sequence of statements to rust code.
    pub fn seq_to_syn(
        stmts: &mut Vec<syn::Stmt>,
        dont_bind: bool,
        crates: &mut BTreeSet<String>,
        vars: Vars,
        subs: Vec<Stmts>,
    ) {
        for sub in subs {
            sub.extend_syn_aux(stmts, crates, false)
        }
        if dont_bind {
            let vars_expr = vars.as_expr().clone().into_syn(crates);
            // println!(
            //     "- `seq_to_syn`, `vars_expr`, don't bind\n  {}",
            //     vars_expr.to_token_stream()
            // );
            stmts.push(syn::Stmt::Expr(vars_expr, None));
            // println!("ok")
        }
    }

    /// Compiles a plain statement to rust code.
    pub fn stmt_to_syn(
        stmts: &mut Vec<syn::Stmt>,
        dont_bind: bool,
        crates: &mut BTreeSet<String>,
        vars: Vars,
        expr: Expr,
    ) {
        let expr = expr.into_syn(crates);
        let stmt = if dont_bind {
            syn::Stmt::Expr(expr, None)
        } else {
            let pat = vars.bind.into_syn();
            // println!(
            //     "- `stmt_to_syn`, do bind\n  pat: {}\n  expr:{}",
            //     pat.to_token_stream(),
            //     expr.to_token_stream(),
            // );
            parse_quote! {
                let #pat = #expr;
            }
        };
        // println!("stmt_to_syn({}, ..): {}", dont_bind, stmt.to_token_stream());
        stmts.push(stmt);
    }

    /// Builds a rayon `Some` result for a fast-rayon branch.
    ///
    /// - `stmts`: the statements of the branch;
    /// - `idx`: the index of the branch;
    /// - `len`: the number of branches.
    ///
    /// See [module-level documentation](self) for details on fast-rayon
    fn rayon_res(stmts: Vec<syn::Stmt>, idx: usize, len: usize) -> syn::Expr {
        let expr: syn::Expr = parse_quote! { Some({#(#stmts)*}) };
        if len == 1 {
            expr
        } else {
            let pref = (0..idx).map::<syn::Expr, _>(|_| parse_quote!(None));
            let suff = ((idx + 1)..len).map::<syn::Expr, _>(|_| parse_quote!(None));
            parse_quote! {
                (#(#pref ,)* #expr #(, #suff)*)
            }
        }
    }

    /// An identifier for the [Option] result of a fast-rayon branch.
    ///
    /// - `idx`: the index of the branch;
    /// - `suff`: a suffix for the identifier, mostly for debugging.
    ///
    /// See [module-level documentation](self) for details on fast-rayon
    fn rayon_opt_ident_with(idx: usize, suff: impl Display) -> syn::Ident {
        syn::Ident::new(
            &format!("reserved_grust_rayon_opt_var_{}{}", idx, suff),
            Span::call_site(),
        )
    }

    /// An identifier for the [Option] result of a fast-rayon branch.
    ///
    /// - `idx`: the index of the branch.
    ///
    /// See [module-level documentation](self) for details on fast-rayon
    fn rayon_opt_ident(idx: usize) -> syn::Ident {
        Self::rayon_opt_ident_with(idx, "")
    }

    /// A [usize] as a rust code literal.
    fn usize_lit(n: usize) -> syn::Lit {
        syn::Lit::Int(syn::LitInt::new(&format!("{}usize", n), Span::call_site()))
    }

    /// The arm of a branch of a fast-rayon pattern-matching.
    ///
    /// - `stmts`: the statements in the branch;
    /// - `idx`: the index of the branch;
    /// - `len`: the number of branches.
    ///
    /// See [module-level documentation](self) for details on fast-rayon
    fn rayon_branch(stmts: Vec<syn::Stmt>, idx: usize, len: usize) -> syn::Arm {
        let pat = syn::Pat::Lit(syn::PatLit {
            attrs: vec![],
            lit: Self::usize_lit(idx),
        });
        let res = Self::rayon_res(stmts, idx, len);
        parse_quote! {
            #pat => #res,
        }
    }

    /// Fast-rayon *run* step rust code for a list of sequences of statements.
    ///
    /// See [module-level documentation](self) for details on fast-rayon
    fn rayon(exprs: impl IntoIterator<Item = Vec<syn::Stmt>> + ExactSizeIterator) -> syn::Stmt {
        let len = exprs.len();
        let len_lit = Self::usize_lit(len);
        let mut ids = Vec::with_capacity(len);
        let mut ids_rgt = Vec::with_capacity(len);
        let mut branches = Vec::with_capacity(len);
        let mut reduce_id: Vec<syn::Expr> = Vec::with_capacity(len);
        let mut reduce_merge: Vec<syn::Expr> = Vec::with_capacity(len);
        let mut tuple_unwrap: Vec<syn::Expr> = Vec::with_capacity(len);
        for (idx, expr) in exprs.into_iter().enumerate() {
            let id = Self::rayon_opt_ident(idx);
            ids.push(id.clone());
            let id_rgt = Self::rayon_opt_ident_with(idx, "_rgt");
            ids_rgt.push(id_rgt.clone());
            branches.push(Self::rayon_branch(expr, idx, len));
            reduce_id.push(parse_quote!(None));
            reduce_merge.push(parse_quote! {
                match (#id, #id_rgt) {
                    (None, None) => None,
                    (Some(val), None) | (None, Some(val)) => Some(val),
                    (Some(_), Some(_)) => unreachable!(
                        "fatal error in rayon reduce operation, found two values"
                    )
                }
            });
            tuple_unwrap.push(parse_quote! {
                #id . expect(
                    "unreachable: fatal error in final rayon unwrap, unexpected `None` value"
                )
            });
        }
        // token_show!(ids, "ids:");
        // token_show!(ids_rgt, "ids_rgt:");
        // token_show!(branches, "branches:");
        // token_show!(reduce_id, "reduce_id:");
        // token_show!(reduce_merge, "reduce_merge:");
        // token_show!(tuple_unwrap, "tuple_unwrap:");
        let ids: syn::Pat = if ids.len() > 1 {
            parse_quote! { (#(#ids),*) }
        } else {
            let id = ids.pop().unwrap();
            parse_quote!(#id)
        };
        let ids_rgt: syn::Pat = if ids_rgt.len() > 1 {
            parse_quote! { (#(#ids_rgt),*) }
        } else {
            let id = ids_rgt.pop().unwrap();
            parse_quote!(#id)
        };
        parse_quote! {{
            let #ids = {
                #[allow(unused_imports)]
                use grust::rayon::prelude::*;
                (0 .. #len_lit)
                    .into_par_iter()
                    .map(|idx: usize| match idx {
                        #(#branches)*
                        idx => unreachable!(
                            "fatal error in rayon branches, illegal index `{}`",
                            idx,
                        ),
                    })
                    .reduce(
                        || ( #(#reduce_id,)* ),
                        |#ids, #ids_rgt| (
                            #(#reduce_merge),*
                        )
                    )
            };
            (
                #(#tuple_unwrap),*
            )
        }}
    }

    /// Compiles a list of parallel statements to a [Schedule] using parallelization kind `Kind`.
    ///
    /// The usual [Vars] value is replaced here with explicit rust versions [syn::Expr] and
    /// [syn::Pat].
    pub fn para_branch_to_syn(
        kind: ParaKind,
        vars_expr: syn::Expr,
        vars_pat: syn::Pat,
        stmts: impl IntoIterator<Item = Vec<syn::Stmt>> + ExactSizeIterator,
    ) -> Schedule {
        let ident = |n: usize| {
            syn::Ident::new(
                &format!("reserved_grust_thread_kid_{}", n),
                Span::call_site(),
            )
        };
        match kind {
            ParaKind::None => {
                // println!("generating branch for `none`");
                let len = stmts.len();
                let stmts = stmts.into_iter().map(|stmts| {
                    let e: syn::Expr = parse_quote! { {#(#stmts)*} };
                    e
                });
                let rhs: syn::Expr = tupleify!(stmts, len);
                let syn = parse_quote! {
                    let #vars_pat = #rhs;
                };
                // let syn = tupleify! {
                //     stmts
                //     => parse_quote! {
                //         let #vars_pat = (#(
                //             {#(#stmts)*}
                //             ,
                //         )*);
                //     }
                //     => parse_quote! {
                //         let #vars_pat = #({#(#stmts)*})*;
                //     }
                // };
                // let syn = if stmts.len() == 1 {
                //     let stmts = stmts.into_iter();
                //     parse_quote! {
                //         let #vars_pat = #({#(#stmts)*})*;
                //     }
                // } else {
                //     let stmts = stmts.into_iter();
                //     parse_quote! {
                //         let #vars_pat = (#(
                //             {#(#stmts)*}
                //             ,
                //         )*);
                //     }
                // };
                Schedule::sequence(vec![syn], vars_expr)
            }
            ParaKind::RayonFast => {
                // println!("generating branch for `rayon-fast` ({})", stmts.len());
                let code = Self::rayon(stmts);
                Schedule::sequence(
                    vec![parse_quote! {
                        let #vars_pat = #code;
                    }],
                    vars_expr,
                )
            }
            ParaKind::Threads => {
                // println!("generating branch for `threads`");
                let (mut stmt_vec, mut return_tuple): (_, Vec<syn::Expr>) = (
                    Vec::with_capacity(stmts.len()),
                    Vec::with_capacity(stmts.len()),
                );
                let scope = Schedule::scope_ident();
                let mut idx = 0;
                for stmt in stmts.into_iter() {
                    let id = ident(idx);
                    idx += 1;
                    // println!("- `para_branch_to_syn`, sub-statements");
                    // println!("  id: {}", id);
                    // println!("  stmt: {}", stmt.to_token_stream());
                    let spawn_let: syn::Stmt = parse_quote! {
                        let #id = #scope . spawn(|| { #(#stmt)* });
                    };
                    // println!("- `para_branch_to_syn`, return tuple element");
                    return_tuple.push(parse_quote! {
                        #id . join().expect("unexpected panic in sub-thread")
                    });
                    stmt_vec.push(spawn_let);
                }
                // println!("- `para_branch_to_syn`, return tuple");
                let return_tuple: syn::Expr = tupleify!(return_tuple);
                // if return_tuple.len() == 1 {
                //     return_tuple.pop().unwrap()
                // } else {
                //     parse_quote! { (#(#return_tuple),*) }
                // };
                Schedule::threads(
                    stmt_vec,
                    vec![parse_quote! {
                        let #vars_pat = #return_tuple;
                    }],
                    vars_expr,
                )
            }
            _ => todo!("unsupported parallelization kind `{}`", kind),
        }
    }

    /// Compiles some parallel statements to rust code.
    pub fn para_to_syn(
        stmts: &mut Vec<syn::Stmt>,
        dont_bind: bool,
        crates: &mut BTreeSet<String>,
        vars: Vars,
        data: Vec<(ParaKind, Vars, Vec<Stmts>)>,
    ) {
        let vars_pat = vars.bind.into_syn();
        let vars_expr = vars.expr.into_syn(crates);
        let mut schedule = Schedule::empty();
        for (kind, vars, subs) in data {
            let vars_expr = vars.expr.into_syn(crates);
            let vars_pat = vars.bind.into_syn();
            let subs = subs.into_iter().map(|sub| sub.into_syn_aux(crates, true));
            let branch_schedule = Self::para_branch_to_syn(kind, vars_expr, vars_pat.clone(), subs);
            schedule.merge(branch_schedule);
        }
        if dont_bind {
            // println!("- `para_to_syn`, don't bind");
            // println!("  spawns:");
            // if let Some(spawns) = schedule.spawn.as_ref() {
            //     for spawn in spawns {
            //         println!("    - {}", spawn.to_token_stream());
            //     }
            // } else {
            //     println!("  - none")
            // }
            // println!("  joins:");
            // if let Some(joins) = schedule.join.as_ref() {
            //     for join in joins {
            //         println!("    - {}", join.to_token_stream());
            //     }
            // } else {
            //     println!("  - none")
            // }
            // println!("  vars_expr: {}", vars_expr.to_token_stream());
            let run = schedule.into_syn(false);
            // println!("  schedule: {}", run.to_token_stream());
            stmts.push(run);
            stmts.push(syn::Stmt::Expr(vars_expr, None));
        } else {
            // println!("- `para_to_syn`, do bind");
            let run = schedule.into_syn(true);
            // println!("  schedule: {}", run.to_token_stream());
            stmts.push(parse_quote! {
                let #vars_pat = #run;
            })
        }
    }

    /// Helper for [Self::extend_syn].
    fn extend_syn_aux(
        self,
        stmts: &mut Vec<syn::Stmt>,
        crates: &mut BTreeSet<String>,
        dont_bind: bool,
    ) {
        match self {
            Self::Seq(vars, subs) => Self::seq_to_syn(stmts, dont_bind, crates, vars, subs),
            Self::Para(vars, subs) => Self::para_to_syn(stmts, dont_bind, crates, vars, subs),
            Self::Stmt(vars, expr) => Self::stmt_to_syn(stmts, dont_bind, crates, vars, expr),
        }
    }

    /// Extends some rust statements with the result of parallel code generation.
    pub fn extend_syn(self, stmts: &mut Vec<syn::Stmt>, crates: &mut BTreeSet<String>) {
        self.extend_syn_aux(stmts, crates, false)
    }

    /// Helper for [Self::into_syn].
    fn into_syn_aux(self, crates: &mut BTreeSet<String>, dont_bind: bool) -> Vec<syn::Stmt> {
        let mut vec = Vec::with_capacity(20);
        self.extend_syn_aux(&mut vec, crates, dont_bind);
        vec
    }

    /// Turns itself into some rust statements.
    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> Vec<syn::Stmt> {
        self.into_syn_aux(crates, false)
    }
}
