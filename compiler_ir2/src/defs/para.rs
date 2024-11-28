prelude! {
    BTreeMap as Map,
    // BTreeSet as Set,
    synced::Synced
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

pub struct Ctx<'a> {
    env: &'a Env<'a>,
}
impl<'a> Ctx<'a> {
    pub fn new(env: &'a Env<'a>) -> Self {
        Self { env }
    }

    const DONT_PARA_WEIGHT_UB: usize = 10;
    const RAYON_PARA_WEIGHT_UB: usize = 20;
}
impl<'a> synced::CtxSpec for Ctx<'a> {
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
    fn instr_cost(&self, uid: usize) -> usize {
        let stmt = self
            .env
            .repr_to_stmt
            .get(&uid)
            .expect("unknown instruction uid");
        use ir1::stream::Kind;
        match &stmt.expr.kind {
            Kind::Expression { expr } => {
                use ir1::expr::Kind;
                match expr {
                    Kind::Identifier { .. } => 0,
                    _ => 10,
                }
            }
            Kind::NodeApplication { .. } => 20,
            Kind::RisingEdge { .. } => 10,
            Kind::FollowedBy { .. } => 0,
            Kind::NoneEvent => 0,
            Kind::SomeEvent { .. } => 0,
        }
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
        let ctx = Ctx::new(self);
        let synced = Synced::new_with(&ctx, &self.graph, subset)?;
        // println!("synced:\n{}", synced);
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn easy_var(id: &str) -> Self {
        Self {
            bind: Pattern::ident(id),
            expr: Expr::ident(id),
        }
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

pub struct Schedule {
    pub spawn: Option<Vec<syn::Stmt>>,
    pub run: Option<Vec<syn::Stmt>>,
    pub join: Option<Vec<syn::Stmt>>,
    pub res: Vec<syn::Expr>,
}
mk_new! { impl Schedule => new {
    spawn: Option<Vec<syn::Stmt>>,
    run: Option<Vec<syn::Stmt>>,
    join: Option<Vec<syn::Stmt>>,
    res: Vec<syn::Expr>,
} }
impl Schedule {
    pub fn empty() -> Schedule {
        Self::new(None, None, None, vec![])
    }
    pub fn threads(spawn: Vec<syn::Stmt>, join: Vec<syn::Stmt>, res: syn::Expr) -> Schedule {
        Self::new(Some(spawn), None, Some(join), vec![res])
    }
    pub fn sequence(seq: Vec<syn::Stmt>, res: syn::Expr) -> Schedule {
        Self::new(None, Some(seq), None, vec![res])
    }

    pub fn next_spawn_index(&self) -> usize {
        self.spawn.as_ref().map(|vec| vec.len()).unwrap_or(0)
    }

    fn scope_ident() -> syn::Ident {
        syn::Ident::new("reserved_grust_thread_scope", Span::call_site())
    }

    fn merge_opt_vec<T>(lft: &mut Option<Vec<T>>, rgt: Option<Vec<T>>) {
        match (lft.as_mut(), rgt) {
            (_, None) => (),
            (None, Some(vec)) => *lft = Some(vec),
            (Some(lft), Some(rgt)) => lft.extend(rgt),
        }
    }
    pub fn merge(&mut self, that: Self) {
        Self::merge_opt_vec(&mut self.spawn, that.spawn);
        Self::merge_opt_vec(&mut self.run, that.run);
        Self::merge_opt_vec(&mut self.join, that.join);
        self.res.extend(that.res);
    }

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
        println!("    body: {}", body.to_token_stream());
        if dont_bind {
            syn::Stmt::Expr(body, None)
        } else {
            parse_quote! {
                let #res = { #body };
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    pub fn easy_seq<'a>(bindings: impl IntoIterator<Item = (&'a str, Expr)>) -> Self {
        Self::new_seq(
            bindings
                .into_iter()
                .map(|(id, expr)| Stmts::stmt(Vars::easy_var(id), expr))
                .collect(),
        )
    }

    pub fn sequential(stmts: &Vec<ir1::stream::Stmt>, syms: &SymbolTable) -> Self {
        Self::new_seq(
            stmts
                .iter()
                .map(|stmt| Self::new_stmt(stmt, syms))
                .collect(),
        )
    }

    pub fn of_ir1(
        stmts: &Vec<ir1::stream::Stmt>,
        syms: &SymbolTable,
        graph: &Graph,
    ) -> Result<Self, String> {
        if conf::component_para().is_none() {
            Ok(Self::sequential(stmts, syms))
        } else {
            let env = Env::new(stmts, graph)?;
            env.print(syms);
            env.to_stmts(syms)
        }
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
                println!("of_synced: para");
                let mut no_para = Vec::with_capacity(map.len());
                let mut no_para_vars = Vec::with_capacity(map.len());
                let mut rayon = Vec::with_capacity(map.len());
                let mut rayon_vars = Vec::with_capacity(map.len());
                let mut threads = Vec::with_capacity(map.len());
                let mut threads_vars = Vec::with_capacity(map.len());
                for (weight, subs) in map {
                    let para_mode = conf::component_para();
                    println!(
                        "para mode: {:?}, weight is {} ({})",
                        para_mode,
                        weight,
                        para_mode.is_rayon(weight < Ctx::RAYON_PARA_WEIGHT_UB)
                    );
                    let (target, target_vars) = if weight < Ctx::DONT_PARA_WEIGHT_UB {
                        (&mut no_para, &mut no_para_vars)
                    } else if para_mode.is_rayon(weight < Ctx::RAYON_PARA_WEIGHT_UB) {
                        (&mut rayon, &mut rayon_vars)
                    } else {
                        (&mut threads, &mut threads_vars)
                    };
                    for sub in subs {
                        let sub = Self::of_synced(env, syms, sub)?;
                        target_vars.push(sub.vars().clone());
                        target.push(sub);
                    }
                }
                println!("ping");
                let mut paras = Vec::with_capacity(3);
                if !no_para.is_empty() {
                    println!("pushing {} no-para(s)", no_para.len());
                    paras.push((
                        ParaKind::None,
                        Vars::merge(no_para_vars.into_iter()),
                        no_para,
                    ));
                }
                if !rayon.is_empty() {
                    println!("pushing {} rayon(s)", rayon.len());
                    paras.push((
                        ParaKind::RayonFast,
                        Vars::merge(rayon_vars.into_iter()),
                        rayon,
                    ));
                }
                if !threads.is_empty() {
                    println!("pushing {} thread(s)", threads.len());
                    paras.push((
                        ParaKind::Threads,
                        Vars::merge(threads_vars.into_iter()),
                        threads,
                    ));
                }
                Ok(Self::new_para(paras))
            }
        }
    }

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
            println!(
                "- `seq_to_syn`, `vars_expr`, don't bind\n  {}",
                vars_expr.to_token_stream()
            );
            stmts.push(syn::Stmt::Expr(vars_expr, None));
            println!("ok")
        }
    }

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
            println!(
                "- `stmt_to_syn`, do bind\n  pat: {}\n  expr:{}",
                pat.to_token_stream(),
                expr.to_token_stream(),
            );
            parse_quote! {
                let #pat = #expr;
            }
        };
        println!("stmt_to_syn({}, ..): {}", dont_bind, stmt.to_token_stream());
        stmts.push(stmt);
    }

    fn rayon_res(stmts: Vec<syn::Stmt>, idx: usize, len: usize) -> syn::Expr {
        let expr: syn::Expr = parse_quote! { Some(#(#stmts)*) };
        let pref = (0..idx).map::<syn::Expr, _>(|_| parse_quote!(None));
        let suff = ((idx + 1)..len).map::<syn::Expr, _>(|_| parse_quote!(None));
        parse_quote! {
            (#(#pref ,)* #expr #(, #suff)*)
        }
    }

    fn rayon_opt_ident_with(idx: usize, suff: impl Display) -> syn::Ident {
        syn::Ident::new(
            &format!("reserved_grust_rayon_opt_var_{}{}", idx, suff),
            Span::call_site(),
        )
    }

    fn rayon_opt_ident(idx: usize) -> syn::Ident {
        Self::rayon_opt_ident_with(idx, "")
    }

    fn usize_lit(n: usize) -> syn::Lit {
        syn::Lit::Int(syn::LitInt::new(&format!("{}usize", n), Span::call_site()))
    }

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
        parse_quote! {{
            let ( #(#ids),* ) = {
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
                        |(#(#ids),*), (#(#ids_rgt),*)| (
                            #(#reduce_merge),*
                        )
                    )
            };
            (
                #(#tuple_unwrap),*
            )
        }}
    }

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
                println!("generating branch for `none`");
                let stmts = stmts.into_iter();
                Schedule::sequence(
                    vec![parse_quote! {
                        let #vars_pat = (#(
                            #(#stmts)*
                            ,
                        )*);
                    }],
                    vars_expr,
                )
            }
            ParaKind::RayonFast => {
                println!("generating branch for `rayon-fast`");
                let code = Self::rayon(stmts);
                Schedule::sequence(
                    vec![parse_quote! {
                        let #vars_pat = #code;
                    }],
                    vars_expr,
                )
            }
            ParaKind::Threads => {
                println!("generating branch for `threads`");
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
                    println!("- `para_branch_to_syn`, return tuple element");
                    return_tuple.push(parse_quote! {
                        #id . join().expect("unexpected panic in sub-thread")
                    });
                    stmt_vec.push(spawn_let);
                }
                println!("- `para_branch_to_syn`, return tuple");
                Schedule::threads(
                    stmt_vec,
                    vec![parse_quote! {
                        let #vars_pat = (#(#return_tuple),*);
                    }],
                    vars_expr,
                )
            }
            _ => todo!("unsupported parallelization kind `{}`", kind),
        }
    }

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
            println!("- `para_to_syn`, don't bind");
            println!("  spawns:");
            if let Some(spawns) = schedule.spawn.as_ref() {
                for spawn in spawns {
                    println!("    - {}", spawn.to_token_stream());
                }
            } else {
                println!("  - none")
            }
            println!("  joins:");
            if let Some(joins) = schedule.join.as_ref() {
                for join in joins {
                    println!("    - {}", join.to_token_stream());
                }
            } else {
                println!("  - none")
            }
            println!("  vars_expr: {}", vars_expr.to_token_stream());
            let run = schedule.into_syn(false);
            println!("  schedule: {}", run.to_token_stream());
            stmts.push(run);
            stmts.push(syn::Stmt::Expr(vars_expr, None));
        } else {
            println!("- `para_to_syn`, do bind");
            let run = schedule.into_syn(true);
            println!("  schedule: {}", run.to_token_stream());
            stmts.push(parse_quote! {
                let #vars_pat = #run;
            })
        }
    }

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

    pub fn extend_syn(self, stmts: &mut Vec<syn::Stmt>, crates: &mut BTreeSet<String>) {
        self.extend_syn_aux(stmts, crates, false)
    }

    fn into_syn_aux(self, crates: &mut BTreeSet<String>, dont_bind: bool) -> Vec<syn::Stmt> {
        let mut vec = Vec::with_capacity(20);
        self.extend_syn_aux(&mut vec, crates, dont_bind);
        vec
    }

    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> Vec<syn::Stmt> {
        self.into_syn_aux(crates, false)
    }
}
