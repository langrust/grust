prelude! {
    BTreeMap as Map,
    // BTreeSet as Set,
    synced::Synced
}

pub struct Ctx;
impl synced::CtxSpec for Ctx {
    type Instr = usize;
    type Cost = usize;
    type Label = graph::Label;
    const INVERTED_EDGES: bool = true;
    fn ignore_edge(label: &Self::Label) -> bool {
        match label {
            graph::Label::Contract => true,
            graph::Label::Weight(w) => *w > 0,
        }
    }
    fn instr_cost(&self, _: usize) -> usize {
        1
    }
    fn sync_seq_cost(&self, seq: &[Synced<Self>]) -> usize {
        seq.len() + seq.iter().map(|s| s.cost()).sum::<usize>()
    }
    fn sync_para_cost(&self, map: &BTreeMap<usize, Vec<Synced<Self>>>) -> usize {
        let (max, count) = map.iter().fold((0, 0), |(max, count), (key, branches)| {
            (std::cmp::max(max, *key), count + branches.len())
        });
        max + count
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParaKind {
    RayonFast,
    Rayon,
    Threads,
    TokioThreads,
}
impl Display for ParaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

pub type Graph = graph::DiGraphMap<usize, graph::Label>;

pub struct Env<'a> {
    id_to_repr: Map<usize, usize>,
    repr_to_stmt: Map<usize, &'a ir1::stream::Stmt>,
    graph: Graph,
}
impl<'a> Env<'a> {
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

    fn saturate_graph(graph: &Graph) -> Graph {
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
                    if is_new {
                        res.add_edge(src, tgt, label.clone());
                        todo.push(tgt);
                    }
                }
            }
        }
        res
    }

    pub fn new(stmts: &'a Vec<ir1::stream::Stmt>, graph: &Graph) -> Result<Self, String> {
        let graph = Self::saturate_graph(graph);
        let mut slf = Self {
            id_to_repr: Map::new(),
            repr_to_stmt: Map::new(),
            graph: Graph::new(),
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

    pub fn repr_of(&self, id: usize) -> Option<usize> {
        self.id_to_repr.get(&id).cloned()
    }

    pub fn to_stmts(&self, syms: &SymbolTable) -> Result<Stmts, String> {
        let subset = self.repr_to_stmt.keys().cloned().collect();
        let synced = Synced::new_with(&Ctx, &self.graph, subset)?;
        println!("synced:\n{}", synced);
        Stmts::of_synced(self, syms, synced)
    }

    pub fn print(&self, syms: &SymbolTable) {
        println!("env:");
        println!("  idents:");
        for tgt in self.id_to_repr.values() {
            let name = syms.get_name(*tgt);
            println!("    {} is {}", tgt, name);
            for (_, tgt, label) in self.graph.edges_directed(*tgt, graph::Direction::Outgoing) {
                println!("      dep {} ({}, {:?})", syms.get_name(tgt), tgt, label);
                for (_, tgt, label) in self.graph.edges_directed(tgt, graph::Direction::Outgoing) {
                    println!("        dep {} ({}, {:?})", syms.get_name(tgt), tgt, label);
                }
            }
        }
        println!("  id_to_repr:");
        for (src, tgt) in &self.id_to_repr {
            println!("    {} ↦ {}", src, tgt);
        }
    }
}

pub type Bindings = ir1::stmt::Pattern;

#[derive(Debug, Clone)]
pub struct Vars {
    bind: Pattern,
    expr: Expr,
}
impl Vars {
    pub fn as_binding(&self) -> &Pattern {
        &self.bind
    }
    pub fn as_expr(&self) -> &Expr {
        &self.expr
    }

    pub fn merge(vars: impl IntoIterator<Item = Self> + ExactSizeIterator) -> Self {
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

    pub fn from_ir1(pat: ir1::stmt::Pattern, syms: &SymbolTable) -> Self {
        let mut curr = pat.kind;
        let mut stack = vec![];

        'current: loop {
            use ir1::stmt::Kind;
            let (mut bind, mut expr) = match curr {
                Kind::Identifier { id } => {
                    let id = syms.get_name(id).clone();
                    (Pattern::ident(id.clone()), Expr::ident(id))
                }
                Kind::Tuple { elements } => {
                    let mut elms = elements.into_iter().map(|pat| pat.kind);
                    curr = elms.next().expect("unexpected empty tuple pattern");
                    stack.push((elms, vec![], vec![]));
                    continue 'current;
                }
                Kind::Typed { id, typ } => {
                    let id = syms.get_name(id).clone();
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

pub enum Stmts {
    Seq(Vars, Vec<Self>),
    Para(Vars, Vec<(ParaKind, Vars, Vec<Self>)>),
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
    pub fn of_ir1(
        stmts: &Vec<ir1::stream::Stmt>,
        syms: &SymbolTable,
        graph: &Graph,
    ) -> Result<Self, String> {
        let env = Env::new(stmts, graph)?;
        env.print(syms);
        env.to_stmts(syms)
    }

    pub fn new_seq(stmts: Vec<Self>) -> Self {
        let vars = Vars::merge(stmts.iter().map(|stmt| stmt.vars().clone()));
        Self::Seq(vars, stmts)
    }
    pub fn new_para(subs: Vec<(ParaKind, Vars, Vec<Self>)>) -> Self {
        let vars = Vars::merge(subs.iter().map(|(_, vars, _)| vars.clone()));
        Self::Para(vars, subs)
    }
    pub fn new_stmt(stmt: &ir1::stream::Stmt, syms: &SymbolTable) -> Self {
        let vars = Vars::from_ir1(stmt.pattern.clone(), syms);
        let expr = stmt.expr.clone().into_ir2(syms);
        Self::Stmt(vars, expr)
    }

    pub fn vars(&self) -> &Vars {
        match self {
            Self::Seq(vars, _) => vars,
            Self::Para(vars, _) => vars,
            Self::Stmt(vars, _) => vars,
        }
    }

    pub fn of_synced(env: &Env, syms: &SymbolTable, synced: Synced<Ctx>) -> Result<Self, String> {
        match synced {
            Synced::Instr(id, _) => {
                let stmt = env
                    .repr_to_stmt
                    .get(&id)
                    .ok_or_else(|| format!("unknown statement identifier `{}`", id))?;
                Ok(Self::new_stmt(stmt, syms))
            }
            Synced::Seq(subs, _) => {
                let mut seq = Vec::with_capacity(subs.len());
                for sub in subs {
                    let sub = Self::of_synced(env, syms, sub)?;
                    seq.push(sub)
                }
                Ok(Self::new_seq(seq))
            }
            Synced::Para(map, _) => {
                let mut para = Vec::with_capacity(map.len());
                let mut vars = Vec::with_capacity(map.len());
                for (_, subs) in map {
                    for sub in subs {
                        let sub = Self::of_synced(env, syms, sub)?;
                        vars.push(sub.vars().clone());
                        para.push(sub);
                    }
                }
                Ok(Self::new_para(vec![(
                    ParaKind::Threads,
                    Vars::merge(vars.into_iter()),
                    para,
                )]))
            }
        }
    }

    pub fn seq_to_syn(
        dont_bind: bool,
        crates: &mut BTreeSet<String>,
        vars: Vars,
        subs: Vec<Stmts>,
    ) -> Vec<syn::Stmt> {
        let mut subs: Vec<_> = subs
            .into_iter()
            .map(|sub| sub.into_syn_aux(crates, false))
            .flatten()
            .collect();
        if dont_bind {
            let vars_expr = vars.as_expr().clone().into_syn(crates);
            println!("- `seq_to_syn`, `vars_expr`, don't bind");
            subs.push(parse_quote! {
                #vars_expr
            });
            subs
        } else {
            subs
        }
    }

    pub fn stmt_to_syn(
        dont_bind: bool,
        crates: &mut BTreeSet<String>,
        vars: Vars,
        expr: Expr,
    ) -> syn::Stmt {
        let expr = expr.into_syn(crates);
        let res = if dont_bind {
            syn::Stmt::Expr(expr, None)
        } else {
            let pat = vars.bind.into_syn();
            println!(
                "- `stmt_to_syn`, do bind\n  pat: {}\n  expr:{}",
                pat.to_token_stream(),
                expr.to_token_stream(),
            );
            parse_quote! {
                let #pat = #expr;
            }
        };
        println!("stmt_to_syn({}, ..): {}", dont_bind, res.to_token_stream());
        res
    }

    pub fn para_branch_to_syn(
        kind: ParaKind,
        vars: Vars,
        stmts: impl IntoIterator<Item = syn::Stmt> + ExactSizeIterator,
    ) -> (Vec<syn::Stmt>, syn::Stmt) {
        let vars_pat = vars.bind.into_syn();
        let ident = |n: usize| {
            syn::Ident::new(
                &format!("reserved_grust_thread_kid_{}", n),
                Span::call_site(),
            )
        };
        match kind {
            ParaKind::Threads => {
                let (mut stmt_vec, mut return_tuple): (_, Vec<syn::Expr>) = (
                    Vec::with_capacity(stmts.len()),
                    Vec::with_capacity(stmts.len()),
                );
                for (idx, stmt) in stmts.into_iter().enumerate() {
                    use quote::ToTokens;
                    let id = ident(idx);
                    println!("- `para_branch_to_syn`, sub-statements");
                    println!("  id: {}", id);
                    println!("  stmt: {}", stmt.to_token_stream());
                    let blah: syn::Stmt = parse_quote! {
                        let #id = std::thread::scope(move || #stmt);
                    };
                    println!("- `para_branch_to_syn`, return tuple element");
                    return_tuple.push(parse_quote! {
                        #id . join().expect("unexpected panic in sub-thread")
                    });
                    stmt_vec.push(blah);
                }
                println!("- `para_branch_to_syn`, return tuple");
                (
                    stmt_vec,
                    parse_quote! {
                        let #vars_pat = (#(#return_tuple),*);
                    },
                )
            }
            _ => todo!("unsupported parallelization kind `{}`", kind),
        }
    }

    pub fn para_to_syn(
        dont_bind: bool,
        crates: &mut BTreeSet<String>,
        vars: Vars,
        data: Vec<(ParaKind, Vars, Vec<Stmts>)>,
    ) -> Vec<syn::Stmt> {
        let vars_pat = vars.bind.into_syn();
        let vars_expr = vars.expr.into_syn(crates);
        let (mut spawns, mut joins) = (
            Vec::with_capacity(data.len()),
            Vec::with_capacity(data.len()),
        );
        for (kind, vars, subs) in data {
            let subs = subs
                .into_iter()
                .map(|stmts| stmts.into_syn_aux(crates, true))
                .flatten()
                .collect::<Vec<_>>();
            let (spawn, join) = Self::para_branch_to_syn(kind, vars, subs.into_iter());
            spawns.extend(spawn);
            joins.push(join);
        }
        if dont_bind {
            println!("- `para_to_syn`, don't bind");
            println!("  spawns:");
            for spawn in &spawns {
                println!("    - {}", spawn.to_token_stream());
            }
            println!("  joins:");
            for join in &joins {
                println!("    {}", join.to_token_stream());
            }
            println!("  vars_expr: {}", vars_expr.to_token_stream());
            spawns.extend(joins);
            spawns.push(syn::Stmt::Expr(vars_expr, None));
            spawns
        } else {
            println!("- `para_to_syn`, do bind");
            vec![parse_quote! {
                let #vars_pat = {
                    #(#spawns)*
                    #(#joins)*
                    #vars_expr
                };
            }]
        }
    }

    pub fn into_syn_aux(self, crates: &mut BTreeSet<String>, dont_bind: bool) -> Vec<syn::Stmt> {
        match self {
            Self::Seq(vars, subs) => Self::seq_to_syn(dont_bind, crates, vars, subs),
            Self::Para(vars, subs) => Self::para_to_syn(dont_bind, crates, vars, subs),
            Self::Stmt(vars, expr) => vec![Self::stmt_to_syn(dont_bind, crates, vars, expr)],
        }
    }

    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> Vec<syn::Stmt> {
        self.into_syn_aux(crates, false)
    }
}
