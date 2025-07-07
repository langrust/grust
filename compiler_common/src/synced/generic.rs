//! Extracts the parallel structure of a flow graph.
//!
//! Takes a [`DiGraphMap`] and a subset of nodes, and generates a [`Synced`] inductive structure
//! corresponding to the most parallel execution of the *"instructions"* inside it.
//!
//!
//!
//! # Algorithm dev documentation
//!
//! > Some tests can be found at the end of this file, they are also available as cargo examples.
//! > Running the examples, *e.g.* `cargo run --example synced3`, yields an execution trace that's
//! > **very** useful for understanding the algorithm.
//!
//! > Also, this discussion mostly ignores the handling of [`CtxSpec::Cost`] for simplicity.
//!
//! First, make sure you understand the [`Synced<Ctx>`] ADT where `Ctx: CtxSpec`. The only relevant
//! parts of [`CtxSpec`] at this point are its `Instr` (instruction indices) and `Cost` (estimated
//! computational cost metric) types. The tests/examples can help as they show the input graph and
//! the resulting [`Synced<_>`] as pseudo-code.
//!
//! The function turning a graph in a [`Synced`] is [`Builder::run`] (actually
//! [`Builder::just_run`]). Structure [`Builder`] stores the data required for `run`, which is
//! introduced bit by bit in this explanation. Most importantly, it stores
//! - the `graph` we're converting,
//! - a set of `nodes` of `graph` which identify the sub-graph we're working on, and
//! - a set `todo` of nodes of `graph` which have yet to be processed.
//!
//! Function `run` is really an outer loop `'find_readies`, which contains another loop `'unstack`.
//! `find_readies` is really responsible for trying to continue a sequence of [`Synced<_>`]. It does
//! so by asking the builder for a list of nodes that are "ready"; a node is "ready" if all its
//! direct dependencies are "validated", meaning we know they have finished running.
//!
//! > When the builder produces ready-nodes, it
//! > - only considers nodes from its `todo` set, and
//! > - removes ready-nodes from the `todo` set before returning.
//!
//! The builder data related to this is
//! - `seq`, the sequence of [`Synced<_>`] under construction, and
//! - `validated`, the nodes validated by `seq`, which is just all the nodes appearing in `seq` in a
//!   [`Synced::Instr`].
//!
//! > To assess whether a node is "ready" the builder also needs to look into the stack, which we
//! > will discuss shortly.
//!
//! Then one of three things can happen:
//! - no node is ready, in which case the current sequence is extracted as a [`Synced::seq`] and we
//!   move on to `unstack`ing.
//! - exactly one node `n` is ready: we add this node in the current sequence (as a
//!   [`Synced::instr`]), register `n` as validated in the current sequence, and loop back on
//!   `find_readies`.
//! - more than one nodes are ready.
//!
//! ## Dealing with parallel branching
//!
//! In this last case, the meaning is that these nodes can/must run in parallel, and for each of
//! them we need to build the sequence of "readies". However, the sequence we are currently building
//! may continue after the parallel branching we just discovered. To deal with this the builder has
//! a `stack` of [`Frame`]s: we push a [`Frame`] containing a [`SyncedFrame::Seq`] with the current
//! `seq` in the builder, **and** the builder's `validated` set (both are [`Vec::drain`]ed): this
//! seq-frame remembers the sequence and the nodes it validates.
//!
//! > The presence of a seq-frame in the stack means that we are building the next [`Synced<_>`] in
//! > the sequence. The [`Frame`]'s `validated` set of the seq-frame corresponds to the nodes
//! > validated by this sequence.
//!
//! Time to deal with the readies; let's call `hd` the head of the "ready-list" and `tl` its
//! (non-empty, by assumption) tail. We push a [`Frame`] containing a [`SyncedFrame::Para`] storing
//! `tl` as "todo nodes" of this parallel branching. That's because we're about to `find_readies`
//! for the sequence starting with `hd`, and will deal with `tl` later. This para-frame only has
//! todo information: it does not validate anything at this point so [`Frame`]'s `validated` is
//! empty.
//!
//! > The presence of a para-frame in the stack means that we are inside one of its parallel
//! > branches.
//!
//! At this point we just put `hd` in the (empty) builder's `seq` (and `validated`) and loop back to
//! `find_readies` to continue `hd`'s sequence.
//!
//! ## Ready and validated
//!
//! Before going over `unstack`, let's go over what validated really means. A node `n` is
//! *validated*, *i.e.* guaranteed to have run "previously", if
//! - `n ∈ validated`, as mentioned above, or
//! - `n ∈ seq_frame.validated` for some **seq**-frame in the `stack`.
//!
//! That's because if a seq-frame is in the stack, then we are building the next step of that
//! sequence and thus `validated` contains all the nodes that are guaranteed to have already run.
//!
//! This is **not** the case for para-frames however, because as we will see shortly the [`Frame`]'s
//! `validated` of a para-frame contains the nodes validated by all parallel branches. But the
//! presence of a para-frame in the stack means we are in one of its branches, thus its validated
//! set is irrelevant because it contains nodes that "run concurrently" to the branch we are in.
//!
//! > The [`Frame`]'s `validated` set of a para-frame contains the nodes validated **after the
//! > parallel branches are joined**, as will become clear below.
//!
//! ## Unstacking
//!
//! The `unstack` inner loop is only reached when there are no "readies", *i.e.* no node can be
//! added to the sequence that `find_readies` was constructing. In this case, we enter `unstack`
//! with a `synced` local variable that's the [`Synced::seq`] of the builder's ([`Vec::drain`]ed)
//! `seq`. Note that `seq` is thus empty, and the builder's `validated` contains the nodes validated
//! by `synced`.
//!
//! The stack is now popped and three things can happen.
//!
//! ### Popping a para-frame
//!
//! We get a para-frame with accumulator `acc` (the parallel branches), list of todo nodes `todo`,
//! and validated set `v`. This means `synced` is a branch of this parallel branching, and we put it
//! in `acc`. We also do `validated.extend(v)` so that the builder's `validated` contains all nodes
//! validated by the para-frame (by the branches in `acc`).
//!
//! Now if `todo` is empty we're done with the para-frame's branches, and we update `synced` to be
//! the [`Synced::para`] of `acc`; then we loop back to `unstack`.
//!
//! If `todo` is not empty call `hd` an element and `tl` the rest. We (re-)push a para-frame with
//! (updated) `acc`, `tl` as the `todo`, and the builder's `validated` set (which we drain). Then we
//! deal with `hd` by putting it in the builder's `seq` and validated before looping back to
//! `find_readies`.
//!
//! ### Popping a seq-frame
//!
//! We get a seq-frame with accumulator `acc` (the sequence so far) and validated set `v`.This means
//! `synced` is the next step in the sequence, and we want to (try to) continue this sequence with
//! `find_readies`. We do `seq.extend(acc)` and `seq.push(synced)` so that the builder's seq
//! contains our sequence, and do `validated.extend(v)` so that its `validated` set now contains all
//! nodes validated by `seq`.
//!
//! Finally we look back to `find_readies`. Easy.
//!
//! ### Popping nothing
//!
//! If the stack is empty, it does **not** mean we're done. It may be that `synced` now validates
//! nodes that allow to build a sequence after `synced`. We detect this by checking whether the
//! builder's `todo` set is empty. (Remember that when producing ready-nodes, the builder removes
//! them from its `todo` set.)
//!
//! > This can loop forever if the graph has a cycle. The algorithm is equipped with cycle
//! > detection, refer to the actual code for details.
//!
//! If `todo` is empty, then we're actually done and the result is the current `Synced`.
//!
//! And that's it.

prelude! {
    BTreeSet as Set,
    BTreeMap as Map,
    graph::{DiGraphMap, Direction},
}

/// Context specification.
///
/// Specifies instruction index/cost types, and how to retrieve costs.
pub trait CtxSpec {
    /// Instruction index type.
    type Instr: Copy + Ord + Display + std::hash::Hash;
    type Label;

    /// Needed by [`Synced`], used during compilation to use different parallelization strategies
    /// for different costs.
    type Cost: Ord + Clone + Display;

    fn ignore_edge(label: &Self::Label) -> bool;

    const INVERTED_EDGES: bool = false;

    /// Yields the cost of an instruction, see [`CtxSpec::Cost`].
    fn instr_cost(&self, i: Self::Instr) -> Self::Cost;

    /// Cost of a sequence of [`Synced`].
    ///
    /// If [`CtxSpec::Cost`] is [`usize`], then a natural implementation of this function is the sum
    /// of the costs of the elements of `seq`.
    fn sync_seq_cost(&self, seq: &[Synced<Self>]) -> Self::Cost;

    /// Cost of a parallel branching of [`Synced`].
    ///
    /// **NB:** the meaning of `(cost, vec) ∈ map` is that **each element of `vec` is a parallel
    /// branch** and its cost is `cost`. Again: `vec` is **not** a sequence, it is a **list of
    /// branches**.
    ///
    /// If [`CtxSpec::Cost`] is [`usize`], then a natural implementation of this function is the max
    /// of the costs of the **flattened** elements of `map`.
    fn sync_para_cost(&self, map: &Map<Self::Cost, Vec<Synced<Self>>>) -> Self::Cost;
}

/// Extends an instruction ADT with a notion of synced instructions.
///
/// Do not use variants directly when constructing values of this type, prefer functions
/// [`Synced::seq`], [`Synced::para`] and [`Synced::instr`]. (Especially the first two which perform
/// basic simplifications.)
///
/// # Examples
///
/// Consider the following graph:
///
/// ```text
/// (0) -> (1, 2) ->  (6, 7) -> (9)
///     \-> (3, 4) -> (8) --/
///      \----> (5) -------/
/// ```
///
/// where
/// - `(i, j, ...)` means instructions `i`, `j`, ... can run concurrently, and
/// - an arrow `a -> b` means `b` must wait for `a`.
///
/// This will be represented as a sequence of three elements `[0, <para>, 9]` where `<para>` encodes
/// the parallel computations between `0` and `9`. The full structure is as follows.
///
/// ```text
/// Seq(
///   Instr(0),
///   Para(
///     Seq( Para(1, 2), Para(6, 7) ),
///     Seq( Para(1, 4), Para(8) ),
///     Instr(5)
///   ),
///   Instr(9),
/// )
/// ```
///
/// Note that in the context of the compiler, the top-level `Seq` is not needed as the compiler
/// builds a `Vec` of instructions anyways.
#[derive(Clone, Debug)]
pub enum Synced<Ctx: CtxSpec + ?Sized> {
    /// A sequence of synced items, running element `i+1` requires first running `i`.
    ///
    /// Do not construct values with this variant directly, use [`Synced::seq`] instead.
    Seq(Vec<Self>, Ctx::Cost),
    /// Maps costs to synced instructions to parallelize.
    ///
    /// Do not construct values with this variant directly, use [`Synced::para`] instead.
    ///
    /// Note that there are 2 levels of parallelization:
    /// - each element `(w, is) ∈ map` represents computations to run in parallel (with `w` the cost
    ///   telling us how), and
    /// - `is` is itself a list of computations to run in parallel (they have the same cost).
    ///
    /// So for instance we could have `map = { w0 ↦ [i1, i2], w1 ↦ [i3, i4] }`; say `w0` (`w1`)
    /// means we must parallelize with rayon (threads). Then `i3` and `i4` will **each** be deployed
    /// in their own thread. While this runs, we run `i1` and `i2` concurrently with rayon. So all
    /// four `i`-s run in parallel.
    ///
    /// # Invariants
    ///
    /// [`Synced::Para(map, cost)`][Synced::Para], with context at time of creation `ctx: Ctx`.
    ///
    /// - `cost = ctx.sync_para_cost(map)`,
    /// - `¬ map.is_empty()`, and
    /// - `(w, is) ∈ map → ¬ is.is_empty()`.
    Para(Map<Ctx::Cost, Vec<Self>>, Ctx::Cost),
    /// Injects a [`CtxSpec::Instr`] in [`Synced<Ctx>`].
    ///
    /// Prefer using [`Synced::instr`] to build values for consistency with the other variants.
    Instr(Ctx::Instr, Ctx::Cost),
}

impl<Ctx: CtxSpec + ?Sized> Display for Synced<Ctx>
where
    Ctx::Instr: Display,
    Ctx::Cost: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seq(seq, w) => {
                write!(f, "Seq({w})[")?;
                let mut first = true;
                for i in seq {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{i}")?;
                }
                write!(f, "]")
            }
            Self::Para(map, w) => {
                write!(f, "Para({w})[")?;
                let mut first = true;
                for (w, is) in map {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{w} ↦ [")?;
                    let mut first = true;
                    for i in is {
                        if first {
                            first = false;
                        } else {
                            write!(f, ", ")?;
                        }
                        write!(f, "{i}")?;
                    }
                    write!(f, "]")?;
                }
                write!(f, "]")
            }
            Self::Instr(i, w) => write!(f, "{i}({w})"),
        }
    }
}

impl<Ctx: CtxSpec + ?Sized> Synced<Ctx> {
    /// Builds a [`Synced`] inductive structure for a sub-graph.
    ///
    /// - `nodes` denotes the nodes of `graph` that are part of the sub-graph; all edges mentioning
    ///   nodes that are not in `nodes` will be ignored.
    pub fn new_with(
        ctx: &Ctx,
        graph: &DiGraphMap<Ctx::Instr, Ctx::Label>,
        nodes: Set<Ctx::Instr>,
    ) -> Result<Self, String> {
        Builder::new_with(graph, nodes).run(ctx)
    }

    /// Builds a [`Synced`] inductive structure for a full graph.
    ///
    /// See [`Synced::new_with`] for sub-graphs.
    pub fn new(ctx: &Ctx, graph: &DiGraphMap<Ctx::Instr, Ctx::Label>) -> Result<Self, String> {
        Builder::new(graph).run(ctx)
    }

    pub fn cost(&self) -> &Ctx::Cost {
        match self {
            Self::Seq(_, w) => w,
            Self::Para(_, w) => w,
            Self::Instr(_, w) => w,
        }
    }
    pub fn seq(mut seq: Vec<Self>, ctx: &Ctx) -> Self {
        if let Some(res) = seq.pop_single() {
            res
        } else {
            let w = ctx.sync_seq_cost(&seq);
            Self::Seq(seq, w)
        }
    }
    pub fn para(mut map: Map<Ctx::Cost, Vec<Self>>, ctx: &Ctx) -> Self {
        if map.len() == 1 {
            for (_w, subs) in map.iter_mut() {
                if let Some(res) = subs.pop_single() {
                    return res;
                }
            }
        }
        let w = ctx.sync_para_cost(&map);
        Self::Para(map, w)
    }
    pub fn instr(i: Ctx::Instr, ctx: &Ctx) -> Self {
        let w = ctx.instr_cost(i);
        Self::Instr(i, w)
    }

    /// Recursive, only for debug.
    pub fn to_pseudo_code(&self, pref: impl AsRef<str>, hide_cost: bool) -> String
    where
        Ctx::Instr: std::fmt::Display,
        Ctx::Cost: std::fmt::Display,
    {
        self.to_pseudo_code_lines(pref, hide_cost)
            .into_iter()
            .fold(String::new(), |mut s, line| {
                if !s.is_empty() {
                    s.push('\n');
                }
                s.push_str(&line);
                s
            })
    }

    /// Recursive, only for debug.
    pub fn to_pseudo_code_lines(&self, pref: impl AsRef<str>, hide_cost: bool) -> Vec<String>
    where
        Ctx::Instr: std::fmt::Display,
        Ctx::Cost: std::fmt::Display,
    {
        let pref = pref.as_ref();
        match self {
            Self::Instr(idx, w) => {
                if hide_cost {
                    vec![format!("{pref}instr#{idx};")]
                } else {
                    vec![format!("{pref}instr#{idx}; //-({w})")]
                }
            }
            Self::Seq(ss, w) => {
                let lines = ss
                    .iter()
                    .flat_map(|s| s.to_pseudo_code_lines(pref, hide_cost));
                let mut res = if hide_cost {
                    vec![]
                } else {
                    vec![format!("{pref}//-[seq]-({w})")]
                };
                res.extend(lines);
                res
            }
            Self::Para(ss, w) => {
                let mut acc = vec![if hide_cost {
                    format!("{pref}join_blocks(")
                } else {
                    format!("{pref}join_blocks( //-({w})")
                }];
                for (w, ss) in ss.iter() {
                    for s in ss {
                        if hide_cost {
                            // Yes, we show the cost of this parallel block if asked to hide costs.
                            // That's because here, `w` is part of the structure of the `Self::Para`
                            // so we want to show it.
                            acc.push(format!("{pref}  ({w})-{{"));
                        } else {
                            // Conversely, if asked to show costs we don't need to show it here as
                            // whatever `s` is, its cost will be visible.
                            acc.push(format!("{pref}  {{"));
                        }
                        acc.extend(s.to_pseudo_code_lines(format!("{pref}    "), hide_cost));
                        acc.push(format!("{pref}  }},"))
                    }
                }
                acc.push(format!("{pref});"));
                acc
            }
        }
    }
}

#[derive(Clone)]
pub enum SyncedFrame<Ctx: CtxSpec + ?Sized> {
    Seq {
        acc: Vec<Synced<Ctx>>,
    },
    Para {
        acc: Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        todo: Vec<Ctx::Instr>,
    },
}
impl<Ctx: CtxSpec + ?Sized> Display for SyncedFrame<Ctx>
where
    Ctx::Instr: Display,
    Ctx::Cost: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seq { acc } => {
                write!(f, "seq[")?;
                let mut first = true;
                for i in acc {
                    if !first {
                        write!(f, ", ")?;
                    } else {
                        first = false;
                    }
                    write!(f, "{i}")?;
                }
                write!(f, "]")
            }
            Self::Para { acc, todo } => {
                write!(f, "para[")?;
                let mut first = true;
                for (w, is) in acc {
                    if !first {
                        write!(f, ", ")?;
                    } else {
                        first = false;
                    }
                    write!(f, "{w} ↦ [")?;
                    let mut first = true;
                    for i in is {
                        if !first {
                            write!(f, ", ")?;
                        } else {
                            first = false;
                        }
                        write!(f, "{i}")?;
                    }
                    write!(f, "]")?;
                }
                write!(f, " | todo: [")?;
                let mut first = true;
                for i in todo {
                    if !first {
                        write!(f, ", ")?;
                    } else {
                        first = false;
                    }
                    write!(f, "{i}")?;
                }
                write!(f, "] ]")
            }
        }
    }
}
impl<Ctx: CtxSpec + ?Sized> SyncedFrame<Ctx> {
    pub fn seq() -> Self {
        // not sure what capacity to use here...
        Self::Seq { acc: Vec::new() }
    }
    pub fn para(todo: Vec<Ctx::Instr>) -> Self {
        Self::Para {
            acc: Map::new(),
            todo,
        }
    }

    pub fn is_seq(&self) -> bool {
        match self {
            Self::Seq { .. } => true,
            Self::Para { .. } => false,
        }
    }
    pub fn is_para(&self) -> bool {
        !self.is_seq()
    }
}

#[derive(Clone)]
pub struct Frame<Ctx: CtxSpec + ?Sized> {
    synced: SyncedFrame<Ctx>,
    validated: Set<Ctx::Instr>,
}
impl<Ctx: CtxSpec + ?Sized> Display for Frame<Ctx>
where
    Ctx::Instr: Display,
    Ctx::Cost: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, validated: [", self.synced)?;
        let mut first = true;
        for i in &self.validated {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }
            write!(f, "{i}")?;
        }
        write!(f, "]")
    }
}
impl<Ctx: CtxSpec + ?Sized> Frame<Ctx> {
    pub fn new(synced: SyncedFrame<Ctx>, validated: Set<Ctx::Instr>) -> Self {
        Self { synced, validated }
    }
    pub fn new_para(todo: Vec<Ctx::Instr>) -> Self {
        Self::new(SyncedFrame::para(todo), Set::new())
    }
    pub fn new_seq(acc: Vec<Synced<Ctx>>, validated: Set<Ctx::Instr>) -> Self {
        Self::new(SyncedFrame::Seq { acc }, validated)
    }

    pub fn is_validated(&self, i: Ctx::Instr) -> bool {
        self.synced.is_seq() && self.validated.contains(&i)
    }

    pub fn merge_validated(&mut self, validated: Set<Ctx::Instr>) {
        self.validated.extend(validated)
    }
}

#[derive(Clone)]
pub struct Stack<Ctx: CtxSpec + ?Sized> {
    stack: Vec<Frame<Ctx>>,
}

impl<Ctx: CtxSpec + ?Sized> std::ops::Deref for Stack<Ctx> {
    type Target = Vec<Frame<Ctx>>;
    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}
impl<Ctx: CtxSpec + ?Sized> Default for Stack<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}
impl<Ctx: CtxSpec + ?Sized> Stack<Ctx> {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(11),
        }
    }

    pub fn is_validated(&self, i: Ctx::Instr) -> bool {
        self.stack.iter().rev().any(|f| f.is_validated(i))
    }

    /// Adds an instruction to the current sequence, and registers `i` as validated.
    ///
    /// Creates a sequence if the top-most element of the stack is not a sequence.
    pub fn seq_add(&mut self, ctx: &Ctx, i: Ctx::Instr) {
        let s = Synced::instr(i, ctx);
        match self.stack.last_mut() {
            Some(Frame {
                synced: SyncedFrame::Seq { acc },
                validated,
            }) => {
                acc.push(s);
                validated.insert(i);
            }
            None
            | Some(Frame {
                synced: SyncedFrame::Para { .. },
                ..
            }) => (),
        }
        let mut validated = BTreeSet::new();
        validated.insert(i);
        self.stack.push(Frame::new(SyncedFrame::seq(), validated))
    }
}

pub trait DebugSpec<'graph, Ctx: CtxSpec + ?Sized> {
    fn debug_init(builder: &Builder<'graph, Ctx>, root_head: Ctx::Instr, root_tail: &[Ctx::Instr]);

    fn find_readies(builder: &Builder<'graph, Ctx>);
    fn find_readies_none(builder: &Builder<'graph, Ctx>);
    fn find_readies_one(builder: &Builder<'graph, Ctx>, i: Ctx::Instr);
    fn find_readies_many(builder: &Builder<'graph, Ctx>, i: Ctx::Instr, is: &[Ctx::Instr]);

    fn unstack(builder: &Builder<'graph, Ctx>, s: &Synced<Ctx>);
    fn unstack_empty(builder: &Builder<'graph, Ctx>);
    fn unstack_empty_todo_nempty(builder: &Builder<'graph, Ctx>);
    fn unstack_empty_todo_empty(builder: &Builder<'graph, Ctx>);
    fn unstack_seq(
        builder: &Builder<'graph, Ctx>,
        acc: &[Synced<Ctx>],
        validated: &Set<Ctx::Instr>,
    );
    fn unstack_para_todo_nempty(
        builder: &Builder<'graph, Ctx>,
        acc: &Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        next: Ctx::Instr,
        todo: &[Ctx::Instr],
        validated: &Set<Ctx::Instr>,
    );
    fn unstack_para_todo_empty(
        builder: &Builder<'graph, Ctx>,
        acc: &Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        validated: &Set<Ctx::Instr>,
    );
}

struct NoDebug;
impl<'graph, Ctx: CtxSpec + ?Sized> DebugSpec<'graph, Ctx> for NoDebug {
    fn debug_init(
        _builder: &Builder<'graph, Ctx>,
        _root_head: Ctx::Instr,
        _root_tail: &[Ctx::Instr],
    ) {
    }

    fn find_readies(_builder: &Builder<'graph, Ctx>) {}

    fn find_readies_none(_builder: &Builder<'graph, Ctx>) {}

    fn find_readies_one(_builder: &Builder<'graph, Ctx>, _i: Ctx::Instr) {}

    fn find_readies_many(_builder: &Builder<'graph, Ctx>, _i: Ctx::Instr, _is: &[Ctx::Instr]) {}

    fn unstack(_builder: &Builder<'graph, Ctx>, _s: &Synced<Ctx>) {}

    fn unstack_empty(_builder: &Builder<'graph, Ctx>) {}

    fn unstack_empty_todo_nempty(_builder: &Builder<'graph, Ctx>) {}

    fn unstack_empty_todo_empty(_builder: &Builder<'graph, Ctx>) {}

    fn unstack_seq(
        _builder: &Builder<'graph, Ctx>,
        _acc: &[Synced<Ctx>],
        _validated: &Set<Ctx::Instr>,
    ) {
    }

    fn unstack_para_todo_nempty(
        _builder: &Builder<'graph, Ctx>,
        _acc: &Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        _next: Ctx::Instr,
        _todo: &[Ctx::Instr],
        _validated: &Set<Ctx::Instr>,
    ) {
    }

    fn unstack_para_todo_empty(
        _builder: &Builder<'graph, Ctx>,
        _acc: &Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        _validated: &Set<Ctx::Instr>,
    ) {
    }
}

pub struct Builder<'graph, Ctx: CtxSpec + ?Sized> {
    graph: &'graph DiGraphMap<Ctx::Instr, Ctx::Label>,
    /// Nodes to consider, allows to run on a sub-graph of `graph`.
    nodes: Set<Ctx::Instr>,
    /// Stack of frames, used in the usual go-down/go-up nested loops in [`Builder::run`].
    stack: Stack<Ctx>,
    /// Instructions left to handle, initialized to all the nodes in the graph by default.
    todo: Set<Ctx::Instr>,
    /// Sequence under construction by the go-down (`'find_readies`) part of [`Builder::run`].
    seq: Vec<Synced<Ctx>>,
    /// Used by [`Builder::run`], set of instructions validated by
    /// - [`Builder::seq`] when going down,
    /// - the `synced: Synced<Ctx>` value we're propagating upwards when going up.
    validated: Set<Ctx::Instr>,
}

impl<'graph, Ctx: CtxSpec + ?Sized> Builder<'graph, Ctx> {
    /// Constructor for the sub-graph of a graph.
    pub fn new_with(
        graph: &'graph DiGraphMap<Ctx::Instr, Ctx::Label>,
        nodes: Set<Ctx::Instr>,
    ) -> Self {
        let todo = nodes.clone();
        Self {
            graph,
            nodes,
            stack: Stack::new(),
            todo,
            seq: Vec::with_capacity(17),
            validated: Set::new(),
        }
    }

    /// Constructor for a state that will run on all the nodes in the graph.
    pub fn new(graph: &'graph DiGraphMap<Ctx::Instr, Ctx::Label>) -> Self {
        Self::new_with(graph, graph.nodes().collect())
    }

    /// Pushes a frame on the stack.
    fn push(&mut self, frame: Frame<Ctx>) {
        self.stack.stack.push(frame)
    }

    /// Pops a frame from the stack.
    fn pop(&mut self) -> Option<Frame<Ctx>> {
        self.stack.stack.pop()
    }

    /// True if `i` is currently validated.
    ///
    /// If true, this means `i` is validated by the sequence under construction, or (partial)
    /// sequences appearing in `self.stack` ---meaning the elements of these sequences come before
    /// the current moment.
    fn is_validated(&self, i: Ctx::Instr) -> bool {
        self.validated.contains(&i) || self.stack.is_validated(i)
    }

    /// Returns the instructions that are ready to be put in the current sequence.
    ///
    /// Used by [`Builder::run`]; there are basically three cases:
    ///
    /// - no instruction is ready: we're done with the current path;
    /// - exactly one instruction is ready: we're simply building a sequence;
    /// - more than one instruction is ready: we're about to run them in parallel.
    fn get_readies(&mut self) -> Option<(Ctx::Instr, Vec<Ctx::Instr>)> {
        let mut res: Option<(Ctx::Instr, Vec<Ctx::Instr>)> = None;
        macro_rules! add {
            { $e:expr } => {
                if let Some((_, tail)) = res.as_mut() {
                    tail.push($e)
                } else { res = Some(($e, vec![])) }
            };
        }
        let direction = if Ctx::INVERTED_EDGES {
            Direction::Outgoing
        } else {
            Direction::Incoming
        };

        // look for instructions that are ready to run
        for i in self.todo.iter().cloned() {
            // `i` is *ready* if all its dependencies are
            if self
                .graph
                .edges_directed(i, direction)
                .all(|(src, tgt, w)| {
                    let dep = if Ctx::INVERTED_EDGES { tgt } else { src };
                    // ignore the edge?
                    Ctx::ignore_edge(w) ||
                    // if `dep` is in the sub-graph, then it must be validated
                    !self.nodes.contains(&dep) || self.is_validated(dep)
                })
            {
                add!(i)
            }
        }

        // update `self.todo`
        if let Some((head, tail)) = res.as_ref() {
            let was_there = self.todo.remove(head);
            debug_assert!(was_there);
            for idx in tail {
                let was_there = self.todo.remove(idx);
                debug_assert!(was_there);
            }
        }

        res
    }

    fn seq_add(&mut self, i: Ctx::Instr, ctx: &Ctx) {
        self.seq.push(Synced::instr(i, ctx));
        self.validated.insert(i);
    }

    fn drain_seq_validated(&mut self) -> Set<Ctx::Instr> {
        std::mem::take(&mut self.validated)
    }

    /// Constructs the [`Synced`] structure corresponding to the internal graph, destroying itself.
    ///
    /// See [module-level documentation][self] for a full discussion of the algorithm.
    ///
    /// # Errors
    ///
    /// - the graph is empty
    /// - the graph is cyclic
    ///
    /// # Panics (debug only)
    ///
    /// - if the internal state does not correspond to [`Builder::new`]/[`Builder::new_with`]
    /// - on internal errors, *i.e.* bugs
    pub fn run(mut self, ctx: &Ctx) -> Result<Synced<Ctx>, String> {
        self.just_run::<NoDebug>(ctx)
    }

    /// Use [`Builder::run`] instead.
    ///
    /// See [module-level documentation][self] for a full discussion of the algorithm.
    pub fn just_run<D: DebugSpec<'graph, Ctx>>(
        &mut self,
        ctx: &Ctx,
    ) -> Result<Synced<Ctx>, String> {
        // sanity check
        debug_assert!(self.stack.is_empty());
        debug_assert_eq!(self.todo.len(), self.nodes.len());
        debug_assert!(self.seq.is_empty());
        debug_assert!(self.validated.is_empty());

        // extract roots for initial setup
        let (root, roots) = self
            .get_readies()
            .ok_or("illegal graph: no root(s) detected, the (sub)graph is empty or cyclic")?;

        D::debug_init(self, root, &roots);

        // if we have other roots, add a parallel frame with `roots` as todo
        if !roots.is_empty() {
            self.push(Frame::new_para(roots))
        }

        // This is used for infinite-loop detection: when `'unstack`-ing, if the stack is empty but
        // `self.todo` is not, we go back to `'find_readies`. If the graph is ill-formed, it can
        // happen that no one is ready and we go back to `'unstack` without doing anything and loop
        // like that forever.
        //
        // By remembering the todo count each time the stack is empty, we can detect this by
        // checking that the count should be strictly decreasing any time the stack is empty.
        let mut previous_todo_count_on_empty_stack = self.todo.len() + 1;

        // start building a sequence starting with `root`
        self.seq_add(root, ctx);

        // look for instructions read to run
        'find_readies: loop {
            // the sequence we're building (`self.seq`) should never be empty at this point
            debug_assert!(!self.seq.is_empty());

            D::find_readies(self);

            let mut synced = match self.get_readies() {
                // nothing is ready, we're done with the current path and need to `'unstack`
                None => {
                    D::find_readies_none(self);
                    Synced::seq(self.seq.drain(0..).collect(), ctx)
                }
                Some((head, tail)) => {
                    if tail.is_empty() {
                        D::find_readies_one(self, head);
                        // a single instruction is ready, add to the current sequence and loop
                        self.seq_add(head, ctx);
                        continue 'find_readies;
                    } else {
                        D::find_readies_many(self, head, &tail);
                        // more than one instruction is ready remember the current sequence
                        {
                            let seq = self.seq.drain(0..).collect();
                            let validated = self.drain_seq_validated();
                            self.push(Frame::new_seq(seq, validated));
                        }
                        // explore the head under a parallel frame
                        self.push(Frame::new_para(tail));
                        self.seq_add(head, ctx);
                        continue 'find_readies;
                    }
                }
            };

            // - only reachable if nothing is ready
            // - so, we need to go up and explore/close other parallel paths to validate more
            //   dependencies
            // - `self.seq` is necessarily empty at this point, it was drained to create `synced`
            debug_assert!(self.seq.is_empty());

            // **do not** change this into a `while` on
            // - `!self.todo.is_empty()`, as `self.stack` might not be empty;
            // - `let Some(frame) = self.stack.pop()`, as `self.todo` might not be empty.
            'unstack: loop {
                D::unstack(self, &synced);

                match self.pop() {
                    None => {
                        D::unstack_empty(self);
                        // actually done?
                        if !self.todo.is_empty() {
                            D::unstack_empty_todo_nempty(self);
                            // have we made any progress since the last time the stack was empty?
                            let todo_count = self.todo.len();
                            if todo_count >= previous_todo_count_on_empty_stack {
                                let mut s = format!(
                                    "ill-formed graph: cycle detected\n\
                                    stack has {} element(s)\
                                    \ntodo:",
                                    self.seq.len(),
                                );
                                for elm in self.seq.iter() {
                                    s = format!("{s}\n- {elm}");
                                }
                                for elm in self.todo.iter() {
                                    s = format!("{s}\n- {elm}");
                                }
                                return Err(s);
                            }
                            previous_todo_count_on_empty_stack = todo_count;
                            // more to do after `synced`, `self.validated` is unchanged
                            self.seq.push(synced);
                            continue 'find_readies;
                        } else {
                            D::unstack_empty_todo_empty(self);
                            // done
                            break 'unstack;
                        }
                    }
                    Some(Frame {
                        synced: SyncedFrame::Seq { acc },
                        validated,
                    }) => {
                        D::unstack_seq(self, &acc, &validated);
                        // we're in a sequence
                        debug_assert!(self.seq.is_empty());
                        self.seq.extend(acc);
                        self.seq.push(synced);
                        self.validated.extend(validated);
                        continue 'find_readies;
                    }
                    Some(Frame {
                        synced: SyncedFrame::Para { mut acc, mut todo },
                        validated,
                    }) => {
                        // just for debug, a bit ugly sorry
                        if let Some(last) = todo.last() {
                            D::unstack_para_todo_nempty(
                                self,
                                &acc,
                                *last,
                                &todo[0..todo.len() - 1],
                                &validated,
                            );
                        } else {
                            D::unstack_para_todo_empty(self, &acc, &validated);
                        }
                        // we're in a parallel branch
                        acc.entry(synced.cost().clone())
                            .or_insert_with(Vec::new)
                            .push(synced);
                        self.validated.extend(validated);
                        // more parallel branches to explore?
                        if let Some(i) = todo.pop() {
                            let validated = self.drain_seq_validated();
                            self.push(Frame::new(SyncedFrame::Para { acc, todo }, validated));
                            self.seq_add(i, ctx);
                            continue 'find_readies;
                        } else {
                            // keep unstacking, `self.validated` is unchanged
                            synced = Synced::para(acc, ctx);
                            continue 'unstack;
                        }
                    }
                }
            }

            debug_assert!(self.stack.is_empty());
            debug_assert!(self.todo.is_empty());
            return Ok(synced);
        }
    }
}

impl<Ctx: CtxSpec + ?Sized> Synced<Ctx> {
    pub fn of_subgraph(
        graph: &DiGraphMap<Ctx::Instr, Ctx::Label>,
        nodes: Set<Ctx::Instr>,
        ctx: &Ctx,
    ) -> Result<Self, String> {
        Builder::new_with(graph, nodes).run(ctx)
    }

    pub fn of_graph(graph: &DiGraphMap<Ctx::Instr, Ctx::Label>, ctx: &Ctx) -> Result<Self, String> {
        Builder::new(graph).run(ctx)
    }
}

#[cfg(debug_assertions)]
pub mod test {
    use super::*;

    use graph::new_graph;

    #[derive(Debug)]
    pub struct DummyCtx;

    impl CtxSpec for DummyCtx {
        type Instr = usize;
        type Cost = usize;
        type Label = usize;
        fn ignore_edge(label: &usize) -> bool {
            *label > 0
        }
        fn instr_cost(&self, i: Self::Instr) -> Self::Cost {
            (i % 3) + 1
        }
        fn sync_seq_cost(&self, seq: &[Synced<Self>]) -> Self::Cost {
            seq.iter().map(Synced::cost).sum()
        }
        fn sync_para_cost(&self, map: &BTreeMap<Self::Cost, Vec<Synced<Self>>>) -> Self::Cost {
            let mut max = 0;
            for c in map.keys() {
                max = std::cmp::max(max, *c);
            }
            max + 1
        }
    }

    impl<'graph> DebugSpec<'graph, DummyCtx> for DummyCtx {
        fn debug_init(
            builder: &Builder<'graph, DummyCtx>,
            root_head: <DummyCtx as CtxSpec>::Instr,
            root_tail: &[<DummyCtx as CtxSpec>::Instr],
        ) {
            println!(
                "starting run with {} todo(s) ({:?}) and {} root(s): ({} :: {:?})",
                builder.todo.len(),
                builder.todo,
                root_tail.len() + 1,
                root_head,
                root_tail,
            );
        }
        fn find_readies(builder: &Builder<'graph, DummyCtx>) {
            println!();
            println!("find_readies");
            println!("- seq: {:?}", builder.seq);
            println!("- seq-validated: {:?}", builder.validated);
            println!("- stack:");
            for frame in builder.stack.iter().rev() {
                println!("  - {frame}")
            }
        }
        fn find_readies_none(_builder: &Builder<'graph, DummyCtx>) {
            println!("- no instruction is ready");
        }
        fn find_readies_one(_builder: &Builder<'graph, DummyCtx>, i: <DummyCtx as CtxSpec>::Instr) {
            println!("- exactly one instruction is ready ({i}), augmenting `seq`")
        }
        fn find_readies_many(
            _builder: &Builder<'graph, DummyCtx>,
            i: <DummyCtx as CtxSpec>::Instr,
            is: &[<DummyCtx as CtxSpec>::Instr],
        ) {
            println!("- readies: {i}, {is:?}");
            println!("- exploring {i} under a parallel frame for the tail");
        }
        fn unstack(builder: &Builder<'graph, DummyCtx>, s: &Synced<DummyCtx>) {
            println!();
            println!("unstacking");
            println!("- synced: {s:?}");
            println!("- validated: {:?}", builder.validated);
            println!("- stack:");
            for frame in builder.stack.iter().rev() {
                println!("  - {frame}")
            }
        }
        fn unstack_empty(_builder: &Builder<'graph, DummyCtx>) {
            println!("- stack is empty")
        }
        fn unstack_empty_todo_empty(_builder: &Builder<'graph, DummyCtx>) {
            println!("- todo is empty, done")
        }
        fn unstack_empty_todo_nempty(builder: &Builder<'graph, DummyCtx>) {
            println!("- todo is not empty: {:?}", builder.todo);
            println!("- going back to finding readies to continue the sequence");
        }
        fn unstack_seq(
            _builder: &Builder<'graph, DummyCtx>,
            acc: &[Synced<DummyCtx>],
            validated: &BTreeSet<<DummyCtx as CtxSpec>::Instr>,
        ) {
            println!("- seq frame {acc:?} with validated {validated:?}");
            println!(
                "- pushing `synced` to sequence, merging validated-s, and back to find readies"
            )
        }
        fn unstack_para_todo_empty(
            _builder: &Builder<'graph, DummyCtx>,
            acc: &BTreeMap<<DummyCtx as CtxSpec>::Cost, Vec<Synced<DummyCtx>>>,
            validated: &BTreeSet<<DummyCtx as CtxSpec>::Instr>,
        ) {
            println!("- parallel frame {acc:?}");
            println!("- validated: {validated:?}");
            println!("- todo is empty");
            println!("- adding `synced`, generating new para `synced` back to unstacking")
        }
        fn unstack_para_todo_nempty(
            _builder: &Builder<'graph, DummyCtx>,
            acc: &BTreeMap<<DummyCtx as CtxSpec>::Cost, Vec<Synced<DummyCtx>>>,
            next: <DummyCtx as CtxSpec>::Instr,
            todo: &[<DummyCtx as CtxSpec>::Instr],
            validated: &BTreeSet<<DummyCtx as CtxSpec>::Instr>,
        ) {
            println!("- parallel frame {acc:?}");
            println!("- todo: {next}, {todo:?}");
            println!("- validated: {validated:?}");
            println!("- adding `synced`, back to finding readies from {next}");
        }
    }

    pub fn run(
        graph: DiGraphMap<usize, usize>,
        graph_pretty: &str,
        expected_pseudo_code: &str,
        sub_graph: Option<&Set<usize>>,
        err_do: impl FnOnce(String),
    ) {
        println!("graph:\n\n```\n{graph_pretty}\n```\n");

        let mut builder = if let Some(sub) = sub_graph {
            Builder::<DummyCtx>::new_with(&graph, sub.clone())
        } else {
            Builder::<DummyCtx>::new(&graph)
        };
        match builder.just_run::<DummyCtx>(&DummyCtx) {
            Ok(s) => {
                let pseudo_code = s.to_pseudo_code("", true);
                if pseudo_code != expected_pseudo_code {
                    println!(
                        "\
                            \n\ngraph:\n\n```\n{graph_pretty}\n```\n\n\
                            node restriction: {sub_graph:?}\n\n\
                            unexpected pseudo-code, \
                            got\n\n```\n{pseudo_code}\n```\n\n\
                            but expected\n\n```\n{expected_pseudo_code}\n```\
                        "
                    );

                    panic!("test failed")
                }

                println!("\n\ngraph:\n\n```\n{graph_pretty}\n```");
                println!("\n\nnode restriction: {sub_graph:?}");
                println!("\n\npseudo-code:\n\n```\n{pseudo_code}\n```");
            }
            Err(e) => {
                err_do(e);
            }
        }
    }

    pub fn run_ok(
        graph: DiGraphMap<usize, usize>,
        graph_pretty: &str,
        expected_pseudo_code: &str,
        sub_graph: Option<&Set<usize>>,
    ) {
        run(graph, graph_pretty, expected_pseudo_code, sub_graph, |e| {
            panic!("{e}")
        });
    }

    #[test]
    fn synced0() {
        run_synced0()
    }

    pub fn run_synced0() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            0 -> 1
            1 -> 2
            2 -> 3
            3 -> 4
        };

        let pretty = "\
0 -> 1 -> 2 -> 3 -> 4\
        ";

        let expected = "\
instr#0;
instr#1;
instr#2;
instr#3;
instr#4;\
        ";

        run_ok(g, pretty, expected, None)
    }

    #[test]
    fn synced1() {
        run_synced1()
    }

    pub fn run_synced1() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            0 -> 1
            0 -> 2
            1 -> 3
            2 -> 3
        };

        let pretty = "\
0 --> 1 ---> 3
  |-> 2 --|\
        ";

        let expected = "\
instr#0;
join_blocks(
  (2)-{
    instr#1;
  },
  (3)-{
    instr#2;
  },
);
instr#3;\
        ";

        run_ok(g, pretty, expected, None)
    }

    #[test]
    fn synced2() {
        run_synced2()
    }

    pub fn run_synced2() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            0 -> 1 0 -> 2
            1 -> 3
            3 -> 4 3 -> 5
            2 -> 4 2 -> 5
            4 -> 6 6 -> 9
            5 -> 7 7 -> 9
            8 -> 9
        };

        let pretty = "\
0 --> 1 ---> 3 -|-> 4 -> 6 -----> 9
  |-> 2 --------|-> 5 -> 7 -|  |
8 -----------------------------|\
        ";

        let expected = "\
join_blocks(
  (3)-{
    instr#8;
  },
  (11)-{
    instr#0;
    join_blocks(
      (3)-{
        instr#1;
        instr#3;
      },
      (3)-{
        instr#2;
      },
    );
    join_blocks(
      (3)-{
        instr#4;
        instr#6;
      },
      (5)-{
        instr#5;
        instr#7;
      },
    );
  },
);
instr#9;\
        ";

        run_ok(g, pretty, expected, None)
    }

    #[test]
    fn synced3() {
        run_synced3()
    }

    pub fn run_synced3() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            0 -> 1 0 -> 2
            1 -> 3 2 -> 3
            3 -> 4 3 -> 5
            4 -> 6 4 -> 7
            7 -> 8
            5 -> 8
            8 -> 9
        };

        let pretty = "\
0 --> 1 ---> 3 ---> 4 -> 6 -> 7 ----> 8 -> 9
  |-> 2 -|      |-------> 5 ------|\
        ";

        let expected = "\
instr#0;
join_blocks(
  (2)-{
    instr#1;
  },
  (3)-{
    instr#2;
  },
);
instr#3;
join_blocks(
  (3)-{
    instr#5;
  },
  (5)-{
    instr#4;
    join_blocks(
      (1)-{
        instr#6;
      },
      (2)-{
        instr#7;
      },
    );
  },
);
instr#8;
instr#9;\
        ";

        run_ok(g, pretty, expected, None)
    }

    #[test]
    fn synced_ignore() {
        run_synced_ignore()
    }

    pub fn run_synced_ignore() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            32 -(0)-> 23
            24 -(0)-> 23
            // 23 -(1)-> 32
        };

        let pretty = "\
32 ----> 23
24 -/
and
23 -(1)-> 32\
        ";

        let expected = "\
join_blocks(
  (1)-{
    instr#24;
  },
  (3)-{
    instr#32;
  },
);
instr#23;\
        ";

        run_ok(g, pretty, expected, None)
    }

    #[test]
    fn synced_cycle() {
        run_synced_cycle()
    }

    pub fn run_synced_cycle() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            0 -> 1
            1 -> 2
            2 -> 3
            3 -> 4
            4 -> 1
            3 -> 5
        };

        let pretty = "\
0 -> 1 -> 2 -> 3 -> 5
     ^--- 4 <--|\
        ";

        let expected = "\
instr#0;
join_blocks(
  (2)-{
    instr#1;
  },
  (3)-{
    instr#2;
  },
);
instr#3;
join_blocks(
  (3)-{
    instr#5;
  },
  (5)-{
    instr#4;
    join_blocks(
      (1)-{
        instr#6;
      },
      (2)-{
        instr#7;
      },
    );
  },
);
instr#8;
instr#9;\
        ";

        let err = "ill-formed graph: cycle detected";
        run(g, pretty, expected, None, |e| {
            if e.starts_with(err) {
                println!("\n\nsuccess: got the expected error `{e}`");
            } else {
                println!("\n\nexpected error `{err}`, got `{e}`");
                panic!("test failed")
            }
        })
    }

    #[test]
    fn synced_sub() {
        run_synced_sub()
    }

    pub fn run_synced_sub() {
        let g: DiGraphMap<usize, usize> = new_graph! {
            0 -> 1 0 -> 2
            1 -> 3 2 -> 3
            3 -> 4 3 -> 5
            4 -> 6 6 -> 7
            7 -> 8
            5 -> 8
            8 -> 9
            // the following is not part of the subgraph
            10 -> 0 10 -> 0
            11 -> 1
            12 -> 5
        };

        let pretty = "\
// only nodes `0` through `9` are part of the subgraph
10 -> 11
|     |
v     v
0 --> 1 ---> 3 ---> 4 -> 6 -> 7 ---> 8 -> 9
  |-> 2 -|     |-------> 5 ------|
                         ^
12-----------------------|
\
        ";

        let expected = "\
instr#0;
join_blocks(
  (2)-{
    instr#1;
  },
  (3)-{
    instr#2;
  },
);
instr#3;
join_blocks(
  (3)-{
    instr#5;
  },
  (5)-{
    instr#4;
    instr#6;
    instr#7;
  },
);
instr#8;
instr#9;\
        ";

        let nodes = (0..=9).collect();
        run_ok(g, pretty, expected, Some(&nodes))
    }
}
