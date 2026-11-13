//! Independent check of a losing search: build a *safe set* of states from which the
//! Algorithm survives forever on boards of at most `max_size` segments.
//!
//! A set `S` of (normalised) states is safe when it contains the start state and every
//! `s` in `S` satisfies:
//! 1. no insert move of `s` leaves the Algorithm without an allowed colour,
//! 2. for every insert move of `s`, some allowed colour leads to a state in `S`,
//! 3. every reduction of `s` is in `S`.
//!
//! At `size(s) ≥ max_size` there are no insert moves, so only rule 3 applies. Such states
//! are not kept in the visited set: when a colour outcome is at the size limit, its
//! reductions are added instead, which is all rule 3 asks of it. The Algorithm makes no
//! choice there, so the visited set (states below the limit) holds all its decisions.
//!
//! The map only guides which colour to pick (`marked_win`): every visited state is checked
//! against the three rules, so a wrong map entry can make the check fail, never pass
//! falsely.

use dashmap::DashSet;
use rayon::prelude::*;
use smallvec::SmallVec;
use std::hash::Hash;

use super::search_state::{FindStatus, SearchState};
use crate::{
    moves::Move,
    state::{compressed::CompressedState, State},
};

/// Visited states per size; sizes are at most 14 (28 events).
type SizeCounts = [usize; 16];

#[derive(Debug)]
pub enum Verification<const MAX_CLIQUE: u32> {
    /// A safe set was found. `by_size[k]`: its states with `k` segments (below the limit).
    Verified { by_size: SizeCounts },
    Failed {
        state: State<MAX_CLIQUE>,
        /// The insert move that broke a rule, if the failure was at an insert move.
        move_: Option<Move>,
        reason: &'static str,
        /// States visited before the failure was found.
        visited: usize,
    },
}

impl<const MAX_CLIQUE: u32> Verification<MAX_CLIQUE> {
    pub fn is_verified(&self) -> bool {
        matches!(self, Verification::Verified { .. })
    }
    pub fn visited(&self) -> usize {
        match self {
            Verification::Verified { by_size } => by_size.iter().sum(),
            Verification::Failed { visited, .. } => *visited,
        }
    }
    /// Visited states per size, up to the largest non-empty size (only when verified).
    pub fn by_size(&self) -> Option<&[usize]> {
        match self {
            Verification::Verified { by_size } => {
                let len = by_size.iter().rposition(|&n| n > 0).map_or(0, |i| i + 1);
                Some(&by_size[..len])
            }
            Verification::Failed { .. } => None,
        }
    }
}

impl<const MAX_CLIQUE: u32> std::fmt::Display for Verification<MAX_CLIQUE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verification::Verified { .. } => {
                write!(f, "verified (safe set of {} states below the size limit", self.visited())?;
                let sizes = self.by_size().unwrap_or_default();
                let sizes: Vec<_> = sizes.iter().enumerate().map(|(k, n)| format!("{k}:{n}")).collect();
                write!(f, "; by size {})", sizes.join(" "))
            }
            Verification::Failed { state, move_, reason, visited } => {
                write!(f, "FAILED at state {state}")?;
                if let Some(Move(start, end)) = move_ {
                    write!(f, ", move {start} {end}")?;
                }
                write!(f, ": {reason} (after {visited} states)")
            }
        }
    }
}

struct Failure<const MAX_CLIQUE: u32> {
    state: State<MAX_CLIQUE>,
    move_: Option<Move>,
    reason: &'static str,
}

/// Key of a normalised state, limits included, in the visited set. Must be injective on
/// the states that are stored: those below the size limit.
trait VisitKey: Copy + Eq + Hash + Send + Sync {
    fn of<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>) -> Self;
}

impl VisitKey for u128 {
    fn of<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>) -> Self {
        state.data
    }
}

/// Compressed events (`5·size − 4` bits, from the top) and the two 5-bit limits (the low
/// 10 bits). The events end with an end code (a 1 bit), which fixes their length. Fits
/// for `size ≤ 11`, so for every state below a size limit of at most 12.
impl VisitKey for u64 {
    fn of<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>) -> Self {
        debug_assert!(state.size() <= MAX_U64_KEY_SIZE, "state too large for a u64 key");
        let events = CompressedState::<8>::new(*state).to_u64();
        events | (state.limit_front() as u64) << 5 | state.limit_back() as u64
    }
}

/// The largest state size whose `u64` key leaves the low 10 bits free: `5·11 − 4 = 51`.
const MAX_U64_KEY_SIZE: u8 = 11;

/// Normalised copy of a state.
fn normalized<const MAX_CLIQUE: u32>(state: State<MAX_CLIQUE>) -> State<MAX_CLIQUE> {
    let mut state = state;
    state.normalize();
    state
}

/// Both reductions of a state (those that exist), normalised.
fn reductions<const MAX_CLIQUE: u32>(
    state: &State<MAX_CLIQUE>,
) -> impl Iterator<Item = State<MAX_CLIQUE>> {
    [state.try_moved_limit_front(), state.try_moved_limit_back()]
        .into_iter()
        .flatten()
        .map(normalized)
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    fn status(&self, state: &State<MAX_CLIQUE>) -> FindStatus {
        self.get_knowledge(state).status
    }

    /// The map says Builder wins from the normalised `state`: a stored `Winning` entry
    /// applies to it or, recursively, to one of its reductions (wins at the size limit are
    /// often stored only for the reductions). The recursion stops at states stored as
    /// losing: their reductions were checked when the search evaluated them. Follows
    /// reductions only, so it ends within `size` levels.
    fn marked_win(&self, state: State<MAX_CLIQUE>) -> bool {
        let mut stack: SmallVec<[State<MAX_CLIQUE>; 32]> = SmallVec::new();
        let mut seen: SmallVec<[u128; 64]> = SmallVec::new();
        stack.push(state);
        while let Some(s) = stack.pop() {
            if seen.contains(&s.data) {
                continue;
            }
            seen.push(s.data);
            match self.status(&s) {
                FindStatus::Winning(_) => return true,
                FindStatus::Losing { .. } => {}
                FindStatus::InProgress | FindStatus::Unknown => stack.extend(reductions(&s)),
            }
        }
        false
    }
}

/// A colour outcome chosen for the safe set: the state itself or, at the size limit, its
/// reductions.
type Targets<const MAX_CLIQUE: u32> = SmallVec<[State<MAX_CLIQUE>; 2]>;

/// Preference among the colour outcomes of a move (lower is better).
type Rank = (u8, u8);
const IN_SAFE_SET: Rank = (0, 0);
const UNKNOWN: Rank = (2, 0);
fn found_losing(depth: u8) -> Rank {
    (1, u8::MAX - depth)
}

struct Verifier<'a, const MAX_CLIQUE: u32, K> {
    search_state: &'a SearchState<MAX_CLIQUE>,
    visited: DashSet<K>,
    max_size: u8,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    /// Tries to build a safe set from this state (see the module docs), layer by layer in
    /// parallel. `search_state` only guides the choice of colours.
    pub fn verify_loss(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        max_size: u8,
    ) -> Verification<MAX_CLIQUE> {
        if max_size <= MAX_U64_KEY_SIZE + 1 {
            Verifier::<MAX_CLIQUE, u64>::new(search_state, max_size).run(*self)
        } else {
            Verifier::<MAX_CLIQUE, u128>::new(search_state, max_size).run(*self)
        }
    }
}

impl<'a, const MAX_CLIQUE: u32, K: VisitKey> Verifier<'a, MAX_CLIQUE, K> {
    fn new(search_state: &'a SearchState<MAX_CLIQUE>, max_size: u8) -> Self {
        Self { search_state, visited: DashSet::new(), max_size }
    }

    fn run(&self, start: State<MAX_CLIQUE>) -> Verification<MAX_CLIQUE> {
        let start = normalized(start);
        let failed = |f: Failure<MAX_CLIQUE>| Verification::Failed {
            state: f.state,
            move_: f.move_,
            reason: f.reason,
            visited: self.visited.len(),
        };
        if self.search_state.marked_win(start) {
            let reason = "the map has a win for the start state";
            return failed(Failure { state: start, move_: None, reason });
        }
        assert!(start.size() < self.max_size, "the start state is at the size limit");
        let mut by_size = SizeCounts::default();
        self.visited.insert(K::of(&start));
        by_size[start.size() as usize] += 1;
        let mut frontier = vec![start];
        while !frontier.is_empty() {
            // Each thread collects the new states and their size counts; no shared counters.
            let layer = frontier
                .par_iter()
                .try_fold(
                    || (Vec::new(), SizeCounts::default()),
                    |(mut new, mut sizes), s| {
                        self.safe_set_children(s, |c| {
                            if self.visited.insert(K::of(&c)) {
                                sizes[c.size() as usize] += 1;
                                new.push(c);
                            }
                        })?;
                        Ok((new, sizes))
                    },
                )
                .try_reduce(
                    || (Vec::new(), SizeCounts::default()),
                    |(mut a, mut sa), (mut b, sb)| {
                        a.append(&mut b);
                        sa.iter_mut().zip(sb).for_each(|(x, y)| *x += y);
                        Ok((a, sa))
                    },
                );
            match layer {
                Ok((new, sizes)) => {
                    by_size.iter_mut().zip(sizes).for_each(|(x, y)| *x += y);
                    frontier = new;
                }
                Err(f) => return failed(f),
            }
        }
        Verification::Verified { by_size }
    }

    /// Passes to `add` the states that rules 2 and 3 put into the safe set for the
    /// normalised state `s` (below the size limit): its reductions, and one colour
    /// outcome per insert move (or that outcome's reductions, when it is at the limit).
    fn safe_set_children(
        &self,
        s: &State<MAX_CLIQUE>,
        mut add: impl FnMut(State<MAX_CLIQUE>),
    ) -> Result<(), Failure<MAX_CLIQUE>> {
        for r in reductions(s) {
            // Not needed for soundness (the reduction would fail when visited), but it
            // gives an earlier and clearer failure.
            if self.search_state.marked_win(r) {
                let reason = "a reduction leads to a state the map marks as winning";
                return Err(Failure { state: *s, move_: None, reason });
            }
            add(r);
        }
        for move_ in s.moves() {
            let Move(start, end) = move_.move_;
            let mut has_colour = false;
            let mut best: Option<(Rank, Targets<MAX_CLIQUE>)> = None;
            for colour in s.allowed_colours_for_segment(start, end) {
                has_colour = true;
                let mut child = *s;
                child.insert_segment(start, end, colour);
                let child = normalized(child);
                if let Some((rank, targets)) = self.assess(child) {
                    if best.as_ref().is_none_or(|(r, _)| rank < *r) {
                        best = Some((rank, targets));
                    }
                }
            }
            if !has_colour {
                let reason = "an insert move leaves the Algorithm without an allowed colour";
                return Err(Failure { state: *s, move_: Some(move_.move_), reason });
            }
            match best {
                Some((_, targets)) => targets.into_iter().for_each(&mut add),
                None => {
                    let reason = "every allowed colour leads to a state the map marks as winning";
                    return Err(Failure { state: *s, move_: Some(move_.move_), reason });
                }
            }
        }
        Ok(())
    }

    /// `None` when the map marks the normalised colour outcome `child` as a win.
    /// Otherwise its rank and what goes into the safe set for it. Preference: already in
    /// the safe set, then found losing by the search (deepest first), then never looked
    /// at. The search stops at the first colour that refutes a move, so most other
    /// colours are unknown, and an unknown one may be a win for Builder.
    fn assess(&self, child: State<MAX_CLIQUE>) -> Option<(Rank, Targets<MAX_CLIQUE>)> {
        let ss = self.search_state;
        if child.size() < self.max_size {
            let rank = match ss.status(&child) {
                FindStatus::Winning(_) => return None,
                FindStatus::Losing { depth } => found_losing(depth),
                FindStatus::InProgress | FindStatus::Unknown => {
                    if ss.marked_win(child) {
                        return None;
                    }
                    UNKNOWN
                }
            };
            let rank = if self.visited.contains(&K::of(&child)) { IN_SAFE_SET } else { rank };
            return Some((rank, SmallVec::from_elem(child, 1)));
        }
        // At the size limit: wins are stored, losses (by default) are not, so it counts
        // as found losing when both its reductions are, at the smaller depth.
        if ss.status(&child).is_winning() {
            return None;
        }
        let targets: Targets<MAX_CLIQUE> = reductions(&child).collect();
        let mut depth = Some(u8::MAX);
        let mut all_visited = true;
        for r in &targets {
            match ss.status(r) {
                FindStatus::Winning(_) => return None,
                FindStatus::Losing { depth: d } => depth = depth.map(|m| m.min(d)),
                FindStatus::InProgress | FindStatus::Unknown => {
                    if ss.marked_win(*r) {
                        return None;
                    }
                    depth = None;
                }
            }
            all_visited &= self.visited.contains(&K::of(r));
        }
        let rank = if all_visited { IN_SAFE_SET } else { depth.map_or(UNKNOWN, found_losing) };
        Some((rank, targets))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::find::FindResult;

    /// Searches depths `3..=depth` (stopping at a win), then verifies the loss.
    fn search_and_verify<const MQ: u32>(max_size: u8, depth: u8) -> Verification<MQ> {
        let _lock = super::super::SEARCH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let search_state = SearchState::<MQ>::new();
        let root = State::<MQ>::new();
        let won = (3..=depth)
            .any(|d| matches!(root.find_strategy(&search_state, d, max_size), FindResult::Winning));
        assert!(!won, "MQ={MQ}, MS={max_size}: the search found a win");
        let result = root.verify_loss(&search_state, max_size);
        println!("MQ={MQ}, MS={max_size}, d={depth}: {result}, map {}", search_state.map.len());
        result
    }

    #[test]
    fn verifies_real_losses() {
        assert!(search_and_verify::<3>(4, 10).is_verified());
        assert!(search_and_verify::<4>(6, 10).is_verified());
        assert!(search_and_verify::<4>(7, 20).is_verified());
    }

    /// Builder wins MQ=3, MS=5 at depth 7 and MQ=4, MS=8 at depth 13: a search cut short
    /// before that must not verify.
    #[test]
    fn rejects_unfinished_searches() {
        assert!(!search_and_verify::<3>(5, 5).is_verified());
        assert!(!search_and_verify::<4>(8, 10).is_verified());
    }

    /// Without any map knowledge, a won game still cannot be verified.
    #[test]
    fn rejects_won_game_with_empty_map() {
        let result = State::<3>::new().verify_loss(&SearchState::<3>::new(), 5);
        assert!(!result.is_verified(), "{result}");
    }

    /// Both key types give the same safe set size and per-size counts.
    #[test]
    fn key_types_agree() {
        let _lock = super::super::SEARCH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let search_state = SearchState::<4>::new();
        let root = State::<4>::new();
        for d in 3..=16 {
            root.find_strategy(&search_state, d, 7);
        }
        // With a fixed map the traversal can still differ between runs (the colour choice
        // depends on which states other threads visited first), so compare one thread.
        let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
        let (a, b) = pool.install(|| {
            (
                Verifier::<4, u64>::new(&search_state, 7).run(root),
                Verifier::<4, u128>::new(&search_state, 7).run(root),
            )
        });
        assert!(a.is_verified() && b.is_verified());
        assert_eq!(a.by_size(), b.by_size());
    }

    /// The `u64` key is injective on reachable states below the size limit (with their
    /// limits): compared with the full `u128` state over a breadth-first sample.
    #[test]
    fn u64_keys_are_injective() {
        let mut keys = std::collections::HashMap::<u64, u128>::new();
        let mut frontier = vec![State::<4>::new()];
        let mut seen = std::collections::HashSet::<u128>::new();
        while let Some(s) = frontier.pop() {
            if seen.len() >= 300_000 || !seen.insert(s.data) {
                continue;
            }
            let previous = keys.insert(u64::of(&s), s.data);
            assert!(previous.is_none_or(|p| p == s.data), "{s} collides");
            let mut next: Vec<State<4>> = reductions(&s).collect();
            if s.size() < MAX_U64_KEY_SIZE {
                for m in s.moves() {
                    let Move(a, b) = m.move_;
                    for c in s.allowed_colours_for_segment(a, b) {
                        let mut child = s;
                        child.insert_segment(a, b, c);
                        next.push(normalized(child));
                    }
                }
            }
            frontier.extend(next);
        }
        assert!(keys.len() >= 300_000);
        assert!(seen.iter().any(|&d| State::<4> { data: d }.size() == MAX_U64_KEY_SIZE));
    }
}
