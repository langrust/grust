//! Hierarchical profiling stats helpers.

prelude! {}

/// Statistics proper.
///
/// Stores a list of triplets composed of
/// - a description for the "task" profiled,
/// - the task's duration,
/// - an optional sub-`Stats` that profiles sub-tasks.
pub struct Stats {
    vec: Vec<(String, Duration, Option<Stats>)>,
}

/// A task description and an start [Instant].
///
/// This is used by
/// - [Stats::start] to create an item, and
/// - [Stats::end] which consumes an item and adds it to the task list ([Stats::vec], private
///   field).
pub struct StatsItem {
    start: Instant,
    desc: String,
}

/// Wraps a [Stats] ref-mut, created by [Stats::as_mut].
///
/// This saves us from writing `&mut Stats` all over the place.
///
/// Note that it is sometimes necessary to use `stats.as_mut()` even when `stats: StatsMut` to
/// accommodate the borrow-checker.
pub struct StatsMut<'a> {
    inner: &'a mut Stats,
}
impl<'a> std::ops::Deref for StatsMut<'a> {
    type Target = Stats;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl<'a> std::ops::DerefMut for StatsMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
impl Stats {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
        }
    }
    pub fn new() -> Self {
        Self::with_capacity(10)
    }

    pub fn as_mut(&mut self) -> StatsMut {
        StatsMut { inner: self }
    }

    pub fn start(&self, desc: impl Into<String>) -> StatsItem {
        StatsItem {
            start: Instant::now(),
            desc: desc.into(),
        }
    }

    pub fn end(&mut self, i: StatsItem) {
        self.vec.push((i.desc, Instant::now() - i.start, None))
    }
    pub fn augment_end(&mut self, i: StatsItem) {
        self.augment(i.desc, Instant::now() - i.start, None)
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn indent(&mut self) {
        for (desc, _, sub_opt) in self.vec.iter_mut() {
            *desc = format!("  {}", desc);
            if let Some(sub) = sub_opt {
                sub.indent()
            }
        }
    }

    /// Looks for a task with the same description and augments its duration and sub-tasks.
    pub fn augment(&mut self, desc: impl Into<String>, time: Duration, sub_opt: Option<Stats>) {
        let desc = desc.into();
        for (desc2, time2, sub_opt2) in &mut self.vec {
            if &desc == desc2 {
                *time2 = time + *time2;
                match (sub_opt, sub_opt2.as_mut()) {
                    (None, _) => (),
                    (Some(sub), None) => *sub_opt2 = Some(sub),
                    (Some(sub), Some(sub2)) => sub2.augment_merge(sub),
                }
                return ();
            }
        }
        self.vec.push((desc, time, sub_opt));
    }

    /// Merges same-description tasks from two [Stats].
    pub fn augment_merge(&mut self, that: Self) {
        for (d, t, s) in that.vec {
            self.augment(d, t, s)
        }
    }

    /// Same as [Stats::augment] but profiles a continuation directly.
    pub fn augment_timed_with<T>(
        &mut self,
        desc: impl Into<String>,
        run: impl FnOnce(StatsMut) -> T,
    ) -> T {
        let mut sub = Self::new();
        let start = Instant::now();
        let res = run(sub.as_mut());
        let time = Instant::now() - start;
        let sub_opt = if sub.is_empty() {
            None
        } else {
            sub.indent();
            Some(sub)
        };
        self.augment(desc, time, sub_opt);
        res
    }

    pub fn timed_with<T>(&mut self, desc: impl Into<String>, run: impl FnOnce(StatsMut) -> T) -> T {
        let start = Instant::now();
        let mut sub = Self::new();
        let res = run(sub.as_mut());
        let sub_opt = if sub.is_empty() {
            None
        } else {
            sub.indent();
            Some(sub)
        };
        self.vec
            .push((desc.into(), Instant::now() - start, sub_opt));
        res
    }

    pub fn timed<T>(&mut self, desc: impl Into<String>, run: impl FnOnce() -> T) -> T {
        self.timed_with(desc, |_| run())
    }

    fn max_key_len(&self) -> usize {
        let mut max = 0;
        for (s, _, sub) in &self.vec {
            max = max.max(s.chars().count());
            if let Some(sub) = sub {
                max = max.max(sub.max_key_len());
            }
        }
        max
    }

    pub fn pretty(&self, conf: &Conf) -> Option<String> {
        let max_depth = conf.stats_depth;
        if max_depth == 0 {
            None
        } else {
            Some(self.pretty_aux(self.max_key_len(), 1, max_depth))
        }
    }

    fn pretty_aux(&self, max_key_len: usize, depth: usize, max_depth: usize) -> String {
        let mut string = String::with_capacity(200);
        let mut sep = "| ";
        for (desc, duration, sub_opt) in self.vec.iter() {
            string.push_str(sep);
            string.push_str(desc);
            for _ in desc.chars().count()..max_key_len {
                string.push(' ');
            }
            string.push_str(" | ");
            let secs = format!("{}.{:0>9}", duration.as_secs(), duration.subsec_nanos());
            for _ in secs.len()..15 {
                string.push(' ');
            }
            string.extend([secs].into_iter());
            string.push_str(" |");
            if let Some(sub) = sub_opt {
                let depth = depth + 1;
                if depth <= max_depth {
                    string.push('\n');
                    string.push_str(&sub.pretty_aux(max_key_len, depth, max_depth));
                }
            }
            sep = "\n| ";
        }
        string
    }
}
