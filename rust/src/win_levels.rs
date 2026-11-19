//! `--win-levels`: the level of every position, and the window property (NEXT_STEPS Task 7).
//!
//! The *game with bound `n`* is the solver's game with `MS = n`. The *level* of a state `s`
//! is the smallest bound `n ≥ size(s)` at which Builder wins from `s` with no depth limit,
//! or 0 if there is none up to `N` (the `-s` value).
//!
//! 1. Enumerate the universe: every state reachable from the empty state by inserts (with
//!    every allowed colour) and reductions at bound `N`, normalised, limits included. States
//!    with `N` segments are not stored; their reductions are.
//! 2. For `n = 1..=N`, sweep the stored states of size `≤ n` with level 0 until nothing
//!    changes, setting `level = n` where Builder wins at bound `n`.
//! 3. Report: universe size per segment count, the empty state's level, Table A ("extra
//!    board helped") and Table B (the window property for each `B0`).
//!
//! This is a separate analysis; it shares only the move generation with the search.

use std::{
    fmt::Write as _,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering::Relaxed},
    time::Instant,
};

use rayon::prelude::*;

use crate::{memory, state::State};

/// The universe at bound `n`: sorted canonical states and their levels.
pub struct Universe<const MAX_CLIQUE: u32> {
    pub n: u8,
    states: Vec<u128>,
    levels: Vec<AtomicU8>,
}

/// The normalised state, and for a symmetric one (whose mirror has the same events but
/// mirrored limits) the smaller of the two, so mirror images share one entry.
#[inline]
fn canonical<const MAX_CLIQUE: u32>(mut s: State<MAX_CLIQUE>) -> State<MAX_CLIQUE> {
    s.normalize();
    if s.is_symmetric() {
        let mut f = s;
        f.flip();
        f.normalize_inner(false);
        if f.data < s.data {
            return f;
        }
    }
    s
}

#[inline]
fn reductions<const MAX_CLIQUE: u32>(
    s: &State<MAX_CLIQUE>,
) -> impl Iterator<Item = State<MAX_CLIQUE>> {
    [s.try_moved_limit_front(), s.try_moved_limit_back()]
        .into_iter()
        .flatten()
}

/// Canonical successors of a stored state `s` (size < `n`) in the enumeration. A child
/// with `n` segments is replaced by its reductions.
fn successors<const MAX_CLIQUE: u32>(s: State<MAX_CLIQUE>, n: u8, out: &mut Vec<u128>) {
    out.extend(reductions(&s).map(|r| canonical(r).data));
    for m in s.moves() {
        for child in m.outcomes() {
            if child.size() >= n {
                out.extend(reductions(&child).map(|r| canonical(r).data));
            } else {
                out.push(canonical(child).data);
            }
        }
    }
}

/// The sorted union of two sorted, duplicate-free slices.
fn union_sorted(a: &[u128], b: &[u128]) -> Vec<u128> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => {
                out.push(a[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                out.push(b[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                out.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
    out
}

/// Merges the sorted `new` (disjoint from `into`) into the sorted `into`, in place.
fn merge_sorted(into: &mut Vec<u128>, new: &[u128]) {
    let old_len = into.len();
    into.resize(old_len + new.len(), 0);
    let (mut i, mut j, mut k) = (old_len, new.len(), into.len());
    while j > 0 {
        k -= 1;
        if i > 0 && into[i - 1] > new[j - 1] {
            into[k] = into[i - 1];
            i -= 1;
        } else {
            into[k] = new[j - 1];
            j -= 1;
        }
    }
}

impl<const MAX_CLIQUE: u32> Universe<MAX_CLIQUE> {
    /// Breadth-first enumeration from the empty state. Levels all start at 0.
    pub fn enumerate(n: u8, verbose: bool) -> Self {
        assert!((1..=14).contains(&n), "bound must be in 1..=14 (28 events)");
        let start = Instant::now();
        let mut visited = vec![canonical(State::<MAX_CLIQUE>::new()).data];
        let mut frontier = visited.clone();
        let mut round = 0;
        while !frontier.is_empty() {
            round += 1;
            // In batches, deduplicated one at a time: collecting the successors of a whole
            // frontier before deduplicating needs ~30x the memory of the result.
            // Sorted runs, merged like a binary counter: each run is less than half the
            // size of the one before it.
            let mut runs: Vec<Vec<u128>> = Vec::new();
            for batch in frontier.chunks(1 << 15) {
                let mut found: Vec<u128> = batch
                    .par_chunks(256)
                    .flat_map_iter(|chunk| {
                        let mut out = Vec::new();
                        for &data in chunk {
                            successors(State::<MAX_CLIQUE> { data }, n, &mut out);
                        }
                        out.sort_unstable();
                        out.dedup();
                        out.retain(|x| visited.binary_search(x).is_err());
                        out
                    })
                    .collect();
                found.par_sort_unstable();
                found.dedup();
                while runs.last().is_some_and(|last| last.len() <= 2 * found.len()) {
                    found = union_sorted(&runs.pop().unwrap(), &found);
                }
                runs.push(found);
            }
            let new = runs
                .into_iter()
                .rev()
                .reduce(|acc, run| union_sorted(&run, &acc))
                .unwrap_or_default();
            merge_sorted(&mut visited, &new);
            if verbose {
                println!(
                    "  round {round:3}: +{:>12}  total {:>12}  ({:.1?})",
                    new.len(),
                    visited.len(),
                    start.elapsed()
                );
            }
            frontier = new;
        }
        visited.shrink_to_fit();
        let levels = (0..visited.len()).map(|_| AtomicU8::new(0)).collect();
        Self {
            n,
            states: visited,
            levels,
        }
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    fn state(&self, i: usize) -> State<MAX_CLIQUE> {
        State { data: self.states[i] }
    }

    fn index(&self, s: State<MAX_CLIQUE>) -> usize {
        let c = canonical(s);
        self.states
            .binary_search(&c.data)
            .unwrap_or_else(|_| panic!("state {c} (from {s}) is not in the universe"))
    }

    fn stored_level(&self, s: State<MAX_CLIQUE>) -> u8 {
        self.levels[self.index(s)].load(Relaxed)
    }

    /// The level of any state of the universe, including unstored ones with `N` segments:
    /// those have only reductions, so their level is `N` if a reduction has a level.
    pub fn level(&self, s: State<MAX_CLIQUE>) -> u8 {
        if s.size() >= self.n {
            if reductions(&s).any(|r| self.stored_level(r) != 0) {
                self.n
            } else {
                0
            }
        } else {
            self.stored_level(s)
        }
    }

    /// Whether `x` is known to win at the current bound (a level is set).
    #[inline]
    fn won(&self, x: State<MAX_CLIQUE>) -> bool {
        self.level(x) != 0
    }

    /// Whether Builder wins from `s` at bound `n`, given the levels known so far.
    fn wins(&self, s: State<MAX_CLIQUE>, n: u8) -> bool {
        reductions(&s).any(|r| self.won(r))
            || (s.size() < n && s.moves().any(|m| m.outcomes().all(|c| self.won(c))))
    }

    /// Step 2: levels for bounds `1..=N`. Returns the number of sweeps per bound.
    pub fn compute_levels(&self, verbose: bool) -> Vec<usize> {
        let start = Instant::now();
        let mut sweeps_per_bound = vec![0];
        for n in 1..=self.n {
            let mut sweeps = 0;
            let mut found = 0;
            loop {
                sweeps += 1;
                let changed = AtomicUsize::new(0);
                (0..self.len()).into_par_iter().for_each(|i| {
                    let s = self.state(i);
                    if s.size() <= n && self.levels[i].load(Relaxed) == 0 && self.wins(s, n) {
                        self.levels[i].store(n, Relaxed);
                        changed.fetch_add(1, Relaxed);
                    }
                });
                let changed = changed.into_inner();
                found += changed;
                if changed == 0 {
                    break;
                }
            }
            if verbose {
                println!(
                    "  bound {n:2}: {found:>12} new winning states, {sweeps} sweeps ({:.1?})",
                    start.elapsed()
                );
            }
            sweeps_per_bound.push(sweeps);
        }
        sweeps_per_bound
    }

    /// Stored states per segment count.
    fn size_counts(&self) -> Vec<usize> {
        let mut counts = vec![0; self.n as usize];
        for i in 0..self.len() {
            counts[self.state(i).size() as usize] += 1;
        }
        counts
    }

    /// Table A: `table[n][size]` = states with `level == n` and `size ≤ n − 1`.
    fn table_a(&self) -> Vec<Vec<usize>> {
        let n_max = self.n as usize;
        let mut table = vec![vec![0usize; n_max]; n_max + 1];
        for i in 0..self.len() {
            let level = self.levels[i].load(Relaxed) as usize;
            let size = self.state(i).size() as usize;
            if level >= 2 && size < level {
                table[level][size] += 1;
            }
        }
        table
    }

    /// The windows of `t` of size `b0`: for each `i`, `i` front reductions then
    /// `size − b0 − i` back reductions; windows needing an impossible reduction are skipped.
    fn windows(t: State<MAX_CLIQUE>, b0: u8) -> impl Iterator<Item = State<MAX_CLIQUE>> {
        let extra = t.size() - b0;
        (0..=extra).filter_map(move |i| {
            let mut w = t;
            for _ in 0..i {
                w = w.try_moved_limit_front()?;
            }
            for _ in 0..extra - i {
                w = w.try_moved_limit_back()?;
            }
            Some(w)
        })
    }

    /// Table B for one `B0`.
    fn table_b_row(&self, b0: u8) -> WindowRow {
        let max_size = (self.n - 1).min(2 * b0 - 2);
        let max_level = self.n.min(2 * b0 - 1);
        (0..self.len())
            .into_par_iter()
            .filter_map(|i| {
                let t = self.state(i);
                let level = self.levels[i].load(Relaxed);
                (b0 < t.size() && t.size() <= max_size && 0 < level && level <= max_level)
                    .then(|| {
                        let ok = Self::windows(t, b0).any(|w| self.level(w) == b0);
                        WindowRow {
                            checked: 1,
                            counterexamples: !ok as usize,
                            smallest: (!ok).then_some((t.size(), t.data)),
                        }
                    })
            })
            .reduce(WindowRow::default, WindowRow::combine)
    }

    /// Step 3: the report, as text.
    pub fn report(&self, list_states: bool) -> String {
        let mut r = String::new();
        let n = self.n as usize;
        let counts = self.size_counts();
        writeln!(r, "Universe (stored states, size < N = {n}): {}", self.len()).unwrap();
        writeln!(r, "  size  states").unwrap();
        for (size, c) in counts.iter().enumerate() {
            writeln!(r, "  {size:4}  {c}").unwrap();
        }
        let empty = self.level(State::new());
        writeln!(r, "\nLevel of the empty state: {empty}").unwrap();

        let mut by_level = vec![0usize; n + 1];
        for l in &self.levels {
            by_level[l.load(Relaxed) as usize] += 1;
        }
        writeln!(r, "\nStates per level (0 = no win up to N):").unwrap();
        for (level, c) in by_level.iter().enumerate() {
            writeln!(r, "  level {level:2}: {c}").unwrap();
        }
        // Every non-empty state reduces step by step to the empty state, so with
        // L = level(empty) != 0, every state s has 0 < level(s) <= max(size(s), L).
        if empty != 0 {
            let breaking = (0..self.len())
                .into_par_iter()
                .filter(|&i| {
                    let level = self.levels[i].load(Relaxed);
                    level == 0 || level > self.state(i).size().max(empty)
                })
                .count();
            writeln!(
                r,
                "  states without 0 < level <= max(size, {empty}): {breaking} (expected 0)"
            )
            .unwrap();
        }

        writeln!(
            r,
            "\nTable A (extra board helped): states with level == n and size <= n-1, by size"
        )
        .unwrap();
        let header: String = (0..n).map(|s| format!(" {:>9}", format!("s={s}"))).collect();
        writeln!(r, "  {:>3} {:>10}{header}", "n", "total").unwrap();
        let table_a = self.table_a();
        let mut largest_helped = None;
        for (level, row) in table_a.iter().enumerate().skip(2) {
            let total: usize = row.iter().sum();
            if total > 0 {
                largest_helped = Some(level);
            }
            let cells: String = row.iter().map(|c| format!(" {c:>9}")).collect();
            writeln!(r, "  {level:>3} {total:>10}{cells}").unwrap();
        }
        writeln!(
            r,
            "  largest n with a non-zero row: {}",
            largest_helped.map_or("none".into(), |l| l.to_string())
        )
        .unwrap();

        writeln!(r, "\nTable B (window property): T with B0 < size(T) <= min(N-1, 2*B0-2), 0 < level(T) <= min(N, 2*B0-1)").unwrap();
        writeln!(
            r,
            "  {:>3} {:>8} {:>9} {:>5} {:>12} {:>15}  smallest counterexample (size, level)",
            "B0", "size<=", "level<=", "full", "checked", "counterexamples"
        )
        .unwrap();
        let mut smallest_ok = None;
        for b0 in MAX_CLIQUE as u8..self.n {
            let row = self.table_b_row(b0);
            let smallest = row.smallest.map_or("-".to_string(), |(size, data)| {
                let s = State::<MAX_CLIQUE> { data };
                format!("{s} ({size}, {})", self.level(s))
            });
            if row.counterexamples == 0 && row.checked > 0 && smallest_ok.is_none() {
                smallest_ok = Some(b0);
            }
            let smallest = if row.checked == 0 { "(vacuous: nothing to check)".into() } else { smallest };
            // A full check of B0 covers sizes up to 2*B0-2 and levels up to 2*B0-1.
            let full = if self.n >= 2 * b0 - 1 { "yes" } else { "no" };
            writeln!(
                r,
                "  {b0:>3} {:>8} {:>9} {full:>5} {:>12} {:>15}  {smallest}",
                (self.n - 1).min(2 * b0 - 2),
                self.n.min(2 * b0 - 1),
                row.checked,
                row.counterexamples
            )
            .unwrap();
        }
        let mq = MAX_CLIQUE as u8;
        writeln!(
            r,
            "  smallest B0 with T checked and no counterexamples: {}   (2(MQ-1)+1 = {}, 2MQ = {}, 3(MQ-1)+1 = {})",
            smallest_ok.map_or("none".into(), |b| b.to_string()),
            2 * (mq - 1) + 1,
            2 * mq,
            3 * (mq - 1) + 1
        )
        .unwrap();

        if list_states {
            writeln!(r, "\nAll stored states (size, level, state):").unwrap();
            let mut all: Vec<_> = (0..self.len()).map(|i| self.state(i)).collect();
            all.sort_by_key(|s| (s.size(), s.to_string()));
            for s in all {
                writeln!(r, "  {} {:2} {s}", s.size(), self.level(s)).unwrap();
            }
        }
        r
    }
}

#[derive(Default)]
struct WindowRow {
    checked: usize,
    counterexamples: usize,
    /// (size, state data) of the smallest counterexample.
    smallest: Option<(u8, u128)>,
}

impl WindowRow {
    fn combine(a: Self, b: Self) -> Self {
        Self {
            checked: a.checked + b.checked,
            counterexamples: a.counterexamples + b.counterexamples,
            smallest: match (a.smallest, b.smallest) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (x, y) => x.or(y),
            },
        }
    }
}

/// The `--win-levels` mode: enumerate, compute levels, print the report.
pub fn run<const MAX_CLIQUE: u32>(n: u8, list_states: bool) {
    println!(
        "Win levels: MQ = {MAX_CLIQUE}, N = {n}, {} threads",
        rayon::current_num_threads()
    );
    let start = Instant::now();
    println!("Enumerating the universe:");
    let universe = Universe::<MAX_CLIQUE>::enumerate(n, true);
    let enumerated = start.elapsed();
    println!("Computing levels:");
    universe.compute_levels(true);
    let levels_done = start.elapsed();
    println!();
    print!("{}", universe.report(list_states));
    let total = start.elapsed();
    let peak = memory::current()
        .peak_rss_bytes
        .map_or("?".to_string(), memory::format_bytes);
    println!(
        "\nTime: enumeration {enumerated:.1?}, levels {:.1?}, report {:.1?}, total {total:.1?}",
        levels_done - enumerated,
        total - levels_done
    );
    println!("Peak RSS: {peak}");
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::find::{search_state::SearchState, FindResult, SEARCH_TEST_LOCK};

    /// The level of the empty state is the smallest board on which Builder wins.
    #[test]
    fn empty_state_levels() {
        for (n, expected) in [(3, 3), (4, 3)] {
            let u = Universe::<2>::enumerate(n, false);
            u.compute_levels(false);
            assert_eq!(u.level(State::new()), expected, "MQ=2, N={n}");
        }
        let u = Universe::<2>::enumerate(2, false);
        u.compute_levels(false);
        assert_eq!(u.level(State::new()), 0, "MQ=2, N=2");
        for (n, expected) in [(4, 0), (5, 5)] {
            let u = Universe::<3>::enumerate(n, false);
            u.compute_levels(false);
            assert_eq!(u.level(State::new()), expected, "MQ=3, N={n}");
        }
    }

    /// Every state the normal solver stores as `Winning` (MQ, MS) has `0 < level ≤ MS` in
    /// the analysis with `N = MS`.
    fn cross_check<const MQ: u32>(ms: u8) {
        let search_state = {
            let _lock = SEARCH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let search_state = SearchState::<MQ>::new();
            let won = (3..=40u8).any(|depth| {
                matches!(
                    State::<MQ>::new().find_strategy(&search_state, depth, ms),
                    FindResult::Winning
                )
            });
            assert!(won, "the solver should win MQ={MQ}, MS={ms}");
            search_state
        };
        let u = Universe::<MQ>::enumerate(ms, false);
        u.compute_levels(false);
        let mut checked = 0;
        for item in search_state.map.iter() {
            for entry in search_state.entries(item.key()) {
                if !matches!(
                    entry.status,
                    crate::find::search_state::FindStatus::Winning(_)
                ) {
                    continue;
                }
                let events = State::<MQ>::from(*item.key());
                let s = events.with_barrier(entry.barrier);
                let level = u.level(s);
                assert!(
                    0 < level && level <= ms,
                    "solver stored {s} as winning, but its level is {level}"
                );
                checked += 1;
            }
        }
        assert!(checked > 100, "too few winning entries: {checked}");
    }

    #[test]
    fn solver_wins_have_levels_mq3() {
        cross_check::<3>(5);
    }

    /// Slower (larger universe); run with `cargo test --release -- --ignored`.
    #[test]
    #[ignore]
    fn solver_wins_have_levels_mq4() {
        cross_check::<4>(8);
    }

    #[test]
    fn merge_sorted_merges() {
        let mut a = vec![1, 4, 9];
        merge_sorted(&mut a, &[0, 2, 5, 10, 11]);
        assert_eq!(a, vec![0, 1, 2, 4, 5, 9, 10, 11]);
        let mut b = vec![];
        merge_sorted(&mut b, &[3]);
        assert_eq!(b, vec![3]);
        assert_eq!(union_sorted(&[1, 3, 5], &[2, 3, 6]), vec![1, 2, 3, 5, 6]);
        assert_eq!(union_sorted(&[], &[2]), vec![2]);
    }
}
