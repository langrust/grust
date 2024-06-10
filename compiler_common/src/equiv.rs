//! Equivalence classes.

prelude! {
    graph::*,
}

safe_index::new! {
    Class,
    map: ClassMap,
}

pub struct Classes<T> {
    classes: ClassMap<(Vec<T>, bool)>,
    deps: DiGraphMap<Class, ()>,
    cache: Vec<T>,
}
impl<T: std::fmt::Display> std::fmt::Display for Classes<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "classes:".fmt(f)?;
        for (idx, (class, stable)) in self.classes.iter().enumerate() {
            let stable = if *stable { " (stable)" } else { "" };
            write!(f, "\n- #{idx}{stable}: [")?;
            for (idx, elm) in class.iter().enumerate() {
                if idx > 0 {
                    ", ".fmt(f)?;
                }
                elm.fmt(f)?;
            }
            "]".fmt(f)?;
        }
        if self.deps.node_count() > 1 {
            "\nedges".fmt(f)?;
            let mut edges = self
                .deps
                .all_edges()
                .map(|(src, tgt, _)| {
                    format!(
                        "\n- [{}, ..]#{} → [{}, ..]#{}",
                        self.classes[src].0[0], src, self.classes[tgt].0[0], tgt,
                    )
                })
                .collect::<Vec<_>>();
            edges.sort();
            for edge in edges {
                edge.fmt(f)?;
            }
        }
        Ok(())
    }
}

macro_rules! dbg {
    {$($stuff:tt)*} => {
        // println!($($stuff)*);
    }
}

impl<T: Eq> Classes<T> {
    fn new_class(&mut self, class: Vec<T>) -> Class {
        let idx = self.classes.push((class, false));
        self.deps.add_node(idx);
        idx
    }

    pub fn new(elems: Vec<T>) -> Self {
        let mut slf = Self {
            classes: ClassMap::with_capacity(elems.len() / 2),
            deps: DiGraphMap::new(),
            cache: Vec::with_capacity(elems.len()),
        };
        slf.new_class(elems);
        slf
    }

    pub fn stabilize(&mut self, is_lt: &impl Fn(&T, &T) -> bool) {
        self.stabilize_linear(is_lt);
        self.stabilize_branches(is_lt);
    }

    pub fn new_stabilized(elems: Vec<T>, is_lt: &impl Fn(&T, &T) -> bool) -> Self {
        let mut slf = Self::new(elems);

        slf.stabilize(is_lt);

        slf
    }

    fn reset_stable_flags(&mut self) {
        for (_, stable) in self.classes.iter_mut() {
            *stable = false;
        }
    }

    fn stabilize_linear(&mut self, is_lt: &impl Fn(&T, &T) -> bool) {
        let mut stable = false;
        while !stable {
            dbg!(
                "stabilizing, {} class(es) with {} edge(s)",
                self.classes.len(),
                self.deps.edge_count(),
            );
            stable = self.stabilize_linear_one(is_lt);
        }
    }

    pub fn stabilize_linear_one(&mut self, is_lt: &impl Fn(&T, &T) -> bool) -> bool {
        debug_assert!(self.cache.is_empty());

        let mut class_above = None;

        // iterate over unstable classes; in addition to the class itself we need
        // - their index `idx` so that we can add edges if needed;
        // - a ref mut to their `stable_flag` to set it to `true` if the class is stable.
        for (idx, class, stable_flag) in
            self.classes
                .index_iter_mut()
                .filter_map(|(idx, (class, is_stable))| {
                    if *is_stable {
                        None
                    } else {
                        Some((idx, class, is_stable))
                    }
                })
        {
            debug_assert!(self.cache.is_empty());

            // Getting technical in here: we're working with indices directly so that we can
            // `Vec::swap_remove` elements if needed
            let mut cnt = 0;

            while cnt < class.len() {
                dbg!("  cnt: {cnt}");
                let remove = class.iter().any(|elem| is_lt(&class[cnt], elem));

                if remove {
                    // note that since `Vec::swap_remove` replace `class[cnt]` with the last
                    // element, we will not increment `cnt` in this branch as `cnt` is already the
                    // index of the next element
                    let elm = class.swap_remove(cnt);
                    self.cache.push(elm);
                } else {
                    cnt += 1;
                }
            }

            if self.cache.is_empty() {
                dbg!("  -> stable");
                *stable_flag = true;
            } else {
                dbg!("  -> unstable");
                debug_assert!(class_above.is_none());
                class_above = Some(idx);
                break;
            }
        }

        if let Some(class_above) = class_above {
            debug_assert!(!self.cache.is_empty());
            let nu_class = self.cache.drain(0..).collect();
            let nu_idx = self.new_class(nu_class);
            self.deps.add_edge(nu_idx, class_above, ());
            false
        } else {
            true
        }
    }

    fn class_is_lt(&self, is_lt: &impl Fn(&T, &T) -> bool, lft: Class, rgt: Class) -> bool {
        self.classes[lft]
            .0
            .iter()
            .any(|lft| self.classes[rgt].0.iter().any(|rgt| is_lt(lft, rgt)))
    }
    fn below_class<'a>(&'a self, idx: Class) -> impl Iterator<Item = Class> + 'a {
        self.deps
            .edges_directed(idx, petgraph::Direction::Incoming)
            .map(|(sub, _, _)| sub)
    }
    fn above_class<'a>(&'a self, idx: Class) -> impl Iterator<Item = Class> + 'a {
        self.deps
            .edges_directed(idx, petgraph::Direction::Outgoing)
            .map(|(_, sup, _)| sup)
    }

    fn drain_not_lt_from<'a>(
        &'a self,
        is_lt: &impl Fn(&T, &T) -> bool,
        elem: &T,
        sub: Class,
        target: &mut Vec<usize>,
    ) {
        for (idx, sub) in self.classes[sub].0.iter().enumerate() {
            if !is_lt(sub, elem) {
                target.push(idx);
            }
        }
    }

    fn drain_not_gt_from<'a>(
        &'a self,
        is_lt: &impl Fn(&T, &T) -> bool,
        elem: &T,
        sup: Class,
        target: &mut Vec<usize>,
    ) {
        for (idx, sup) in self.classes[sup].0.iter().enumerate() {
            if !is_lt(elem, sup) {
                target.push(idx);
            }
        }
    }

    fn add_lowest_edges(
        &mut self,
        is_lt: &impl Fn(&T, &T) -> bool,
        class: Class,
        mut below: Vec<Class>,
    ) {
        println!("add lowest {class}");
        while let Some(sub) = below.pop() {
            if self.class_is_lt(is_lt, sub, class) {
                println!("- {sub} is lt");
                self.deps.add_edge(sub, class, ());
            } else {
                println!("- {sub} is not lt");
                below.extend(self.below_class(sub))
            }
        }
    }

    fn add_highest_edges(
        &mut self,
        is_lt: &impl Fn(&T, &T) -> bool,
        class: Class,
        mut above: Vec<Class>,
    ) {
        println!("add highest {class}");
        while let Some(sup) = above.pop() {
            if self.class_is_lt(is_lt, class, sup) {
                println!("- {sup} is gt");
                self.deps.add_edge(class, sup, ());
            } else {
                println!("- {sup} is not gt");
                above.extend(self.above_class(sup))
            }
        }
    }

    fn stabilize_branches(&mut self, is_lt: &impl Fn(&T, &T) -> bool) {
        self.reset_stable_flags();
        let mut stable = false;
        while !stable {
            stable = self.stabilize_branches_one(is_lt);
        }
    }

    fn stabilize_branches_one(&mut self, is_lt: &impl Fn(&T, &T) -> bool) -> bool {
        let mut todo: Option<(Vec<T>, Class, bool)> = None;
        let mut not_lt: Vec<usize> = vec![];
        let mut not_lt_cache: Vec<usize> = vec![];

        // iterate on class `idx`-s to avoid borrowing `self` at all
        'find_subclass_to_branch: for idx in self.classes.indices() {
            dbg!("\n- #{idx}");
            if self.classes[idx].1 {
                continue 'find_subclass_to_branch;
            }
            let mut cnt = 0;

            // find all elements `e` in `class` such that all elements `e'` in the classes below are
            // such that `!is_lt(e', e)`
            'find_todo: while cnt < self.classes[idx].0.len() {
                let elem = &self.classes[idx].0[cnt];
                debug_assert!(not_lt_cache.is_empty());

                if let Some((todo, class, below)) = todo.as_mut() {
                    let class = *class;
                    if *below {
                        self.drain_not_lt_from(is_lt, elem, class, &mut not_lt_cache);
                        if not_lt_cache == not_lt {
                            todo.push(self.classes[idx].0.swap_remove(cnt));
                            not_lt_cache.clear();
                            continue 'find_todo;
                        } else {
                            not_lt_cache.clear();
                        }
                    } else {
                        self.drain_not_gt_from(is_lt, elem, class, &mut not_lt_cache);
                        if not_lt_cache == not_lt {
                            todo.push(self.classes[idx].0.swap_remove(cnt));
                            not_lt_cache.clear();
                            continue 'find_todo;
                        } else {
                            not_lt_cache.clear();
                        }
                    }
                } else {
                    for (sub, _, _) in self.deps.edges_directed(idx, petgraph::Direction::Incoming)
                    {
                        debug_assert!(not_lt_cache.is_empty());
                        self.drain_not_lt_from(is_lt, elem, sub, &mut not_lt_cache);
                        if !not_lt_cache.is_empty() {
                            todo = Some((vec![self.classes[idx].0.swap_remove(cnt)], sub, true));
                            not_lt.extend(not_lt_cache.drain(0..));
                            continue 'find_todo;
                        }
                    }
                    for (_, sup, _) in self.deps.edges_directed(idx, petgraph::Direction::Outgoing)
                    {
                        debug_assert!(not_lt_cache.is_empty());
                        self.drain_not_gt_from(is_lt, elem, sup, &mut not_lt_cache);
                        if !not_lt_cache.is_empty() {
                            todo = Some((vec![self.classes[idx].0.swap_remove(cnt)], sup, false));
                            not_lt.extend(not_lt_cache.drain(0..));
                            continue 'find_todo;
                        }
                    }
                }
                cnt += 1;
            }

            if let Some((nu_class, pivot, below)) = std::mem::replace(&mut todo, None) {
                println!(
                    "- trying to {}-pivot on #{} (#{})",
                    if below { "down" } else { "up" },
                    idx,
                    pivot
                );
                if self.classes[idx].0.is_empty() {
                    // drained the class, add back the elements we removed
                    self.classes[idx].0.extend(nu_class);
                    // we can remove the edge from/to `pivot` if `not_lt` contains all
                    // so the `idx`-edge from/to `pivot` is not useful, just remove it
                    if self.classes[pivot].0.len() == not_lt.len() {
                        if below {
                            println!("  pivoting, removing #{pivot} → #{idx}");
                            self.deps.remove_edge(pivot, idx);
                        } else {
                            println!("  pivoting, removing #{idx} → #{pivot}");
                            self.deps.remove_edge(idx, pivot);
                        }
                        return false;
                    } else {
                        println!("  not pivoting");
                        not_lt.clear();
                    }
                } else {
                    // new class, register and add same above/below-deps as `idx`
                    let nu_idx = self.new_class(nu_class);
                    if below {
                        for above in self.above_class(idx).collect::<Vec<_>>() {
                            let _prev = self.deps.add_edge(nu_idx, above, ());
                            debug_assert!(_prev.is_none());
                        }
                        let below = self.below_class(idx).collect();
                        self.add_lowest_edges(is_lt, nu_idx, below);
                    } else {
                        for below in self.below_class(idx).collect::<Vec<_>>() {
                            let _prev = self.deps.add_edge(below, nu_idx, ());
                            debug_assert!(_prev.is_none());
                        }
                        let above = self.above_class(idx).collect();
                        self.add_highest_edges(is_lt, nu_idx, above);
                    }
                    return false;
                }
            }
            self.classes[idx].1 = true;
        }

        true
    }
}

#[test]
fn equiv_test_1() {
    let elems = (0..10).collect();

    // (0) -> (1, 2)  -> (6, 7) -> (9)
    //     \-> (3, 4) -> (8) --/
    //      \----> (5) -------/
    let map: Vec<HashSet<_>> = vec![
        // 0 ->
        [1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter().collect(),
        // 1 ->
        [6, 7, 9].into_iter().collect(),
        // 2 ->
        [6, 7, 9].into_iter().collect(),
        // 3 ->
        [8, 9].into_iter().collect(),
        // 4 ->
        [8, 9].into_iter().collect(),
        // 5 ->
        [9].into_iter().collect(),
        // 6 ->
        [9].into_iter().collect(),
        // 7 ->
        [9].into_iter().collect(),
        // 8 ->
        [9].into_iter().collect(),
        // 9 ->
        [].into_iter().collect(),
    ];

    let is_lt = |lft: usize, rgt: usize| {
        // println!("  is_lt({lft}, {rgt})");
        map[lft].contains(&rgt)
    };

    println!("running on elements {elems:?}");
    let mut classes = Classes::new(elems);
    let mut stable = false;
    println!("\n---------------\n\nline-stabilizing...\n");
    while !stable {
        println!("---");
        println!("{classes}");
        stable = classes.stabilize_linear_one(&|lft, rgt| is_lt(*lft, *rgt));
    }
    println!("\n---\n");
    println!("linear stabilization result:\n{classes}");

    println!("\n---------------\n\nbranch-stabilizing...\n");
    classes.reset_stable_flags();
    stable = false;
    while !stable {
        println!("---");
        println!("{classes}");
        stable = classes.stabilize_branches_one(&|lft, rgt| is_lt(*lft, *rgt));
    }
    println!("\n---\n");

    println!("result:\n{classes}");

    let expected = r#"
classes:
- #0 (stable): [9]
- #1 (stable): [8]
- #2 (stable): [4, 3]
- #3 (stable): [0]
- #4 (stable): [5]
- #5 (stable): [7, 6]
- #6 (stable): [1, 2]
edges
- [0, ..]#3 → [1, ..]#6
- [0, ..]#3 → [4, ..]#2
- [0, ..]#3 → [5, ..]#4
- [1, ..]#6 → [7, ..]#5
- [1, ..]#6 → [9, ..]#0
- [4, ..]#2 → [8, ..]#1
- [5, ..]#4 → [9, ..]#0
- [7, ..]#5 → [9, ..]#0
- [8, ..]#1 → [9, ..]#0
    "#;
    let result = classes.to_string();
    if result != expected.trim() {
        eprintln!("test failed, expected\n\n```{expected}```\n\ngot\n\n```\n{result}\n```");
        panic!("test failed, see output")
    }
}

#[test]
fn equiv_test_2() {
    let elems = (0..7).collect();

    // (0) -----> (1) -----> (3) -----> (6)
    //     \--> (2) -> (4) -> (5) --/
    let map: Vec<HashSet<_>> = vec![
        // 0 ->
        [1, 2, 3, 4, 5, 6].into_iter().collect(),
        // 1 ->
        [3, 6].into_iter().collect(),
        // 2 ->
        [4, 5, 6].into_iter().collect(),
        // 3 ->
        [6].into_iter().collect(),
        // 4 ->
        [5, 6].into_iter().collect(),
        // 5 ->
        [6].into_iter().collect(),
        // 6 ->
        [].into_iter().collect(),
    ];

    let is_lt = |lft: usize, rgt: usize| {
        // println!("  is_lt({lft}, {rgt})");
        map[lft].contains(&rgt)
    };

    println!("running on elements {elems:?}");
    let mut classes = Classes::new(elems);
    let mut stable = false;
    println!("\n---------------\n\nline-stabilizing...\n");
    while !stable {
        println!("---");
        println!("{classes}");
        stable = classes.stabilize_linear_one(&|lft, rgt| is_lt(*lft, *rgt));
    }
    println!("\n---\n");
    println!("linear stabilization result:\n{classes}");

    println!("\n---------------\n\nbranch-stabilizing...\n");
    classes.reset_stable_flags();
    stable = false;
    while !stable {
        println!("---");
        println!("{classes}");
        stable = classes.stabilize_branches_one(&|lft, rgt| is_lt(*lft, *rgt));
    }
    println!("\n---\n");

    println!("result:\n{classes}");

    let expected = r#"
classes:
- #0 (stable): [6]
- #1 (stable): [5]
- #2 (stable): [1]
- #3 (stable): [2]
- #4 (stable): [0]
- #5 (stable): [3]
- #6 (stable): [4]
edges
- [0, ..]#4 → [2, ..]#3
- [1, ..]#2 → [3, ..]#5
- [2, ..]#3 → [4, ..]#6
- [3, ..]#5 → [6, ..]#0
- [4, ..]#6 → [5, ..]#1
- [4, ..]#6 → [6, ..]#0
- [5, ..]#1 → [6, ..]#0
    "#;
    let result = classes.to_string();
    if result != expected.trim() {
        eprintln!("test failed, expected\n\n```{expected}```\n\ngot\n\n```\n{result}\n```");
        panic!("test failed, see output")
    }
}

#[test]
fn equiv_test_3() {
    let elems = (0..=6).collect();

    // (0) -> (1, 2) --> (5) ---> (6)
    //      \----> (3, 4) ----/
    let map: Vec<HashSet<_>> = vec![
        // 0 ->
        (1..=6).collect(),
        // 1 ->
        [5, 6].into_iter().collect(),
        // 2 ->
        [5, 6].into_iter().collect(),
        // 3 ->
        [6].into_iter().collect(),
        // 4 ->
        [6].into_iter().collect(),
        // 5 ->
        [6].into_iter().collect(),
        // 6 ->
        [].into_iter().collect(),
    ];

    let is_lt = |lft: usize, rgt: usize| {
        // println!("  is_lt({lft}, {rgt})");
        map[lft].contains(&rgt)
    };

    println!("running on elements {elems:?}");
    let mut classes = Classes::new(elems);
    let mut stable = false;
    println!("\n---------------\n\nline-stabilizing...\n");
    while !stable {
        println!("---");
        println!("{classes}");
        stable = classes.stabilize_linear_one(&|lft, rgt| is_lt(*lft, *rgt));
    }
    println!("\n---\n");
    println!("linear stabilization result:\n{classes}");

    println!("\n---------------\n\nbranch-stabilizing...\n");
    classes.reset_stable_flags();
    stable = false;
    while !stable {
        println!("---");
        println!("{classes}");
        stable = classes.stabilize_branches_one(&|lft, rgt| is_lt(*lft, *rgt));
    }
    println!("\n---\n");

    println!("result:\n{classes}");

    let expected = r#"
classes:
- #0 (stable): [6]
- #1 (stable): [5]
- #2 (stable): [1, 2]
- #3 (stable): [0]
- #4 (stable): [3, 4]
edges
- [0, ..]#3 → [1, ..]#2
- [0, ..]#3 → [3, ..]#4
- [1, ..]#2 → [5, ..]#1
- [3, ..]#4 → [6, ..]#0
- [5, ..]#1 → [6, ..]#0
    "#;
    let result = classes.to_string();
    if result != expected.trim() {
        eprintln!("test failed, expected\n\n```{expected}```\n\ngot\n\n```\n{result}\n```");
        panic!("test failed, see output")
    }
}
