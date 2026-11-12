use crate::{
    moves::StateWithMove,
    state::State,
    stats::{self, Caller, Counter, Lookup},
};
use rand::seq::SliceRandom;
use rayon::prelude::*;
pub mod root;

pub mod search_state;

use search_state::*;

#[derive(Debug, Clone, Copy)]
pub enum FindResult {
    Winning,
    Losing,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
    ) -> FindResult {
        self.find_strategy_from(search_state, depth, max_size, Caller::Root)
    }
    /// `find_strategy`, recording `caller` in the search statistics.
    fn find_strategy_from(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
        caller: Caller,
    ) -> FindResult {
        let mut norm = *self;
        norm.normalize();
        norm.find_strategy_inner(search_state, depth, max_size, caller)
    }
    fn find_strategy_inner(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
        caller: Caller,
    ) -> FindResult {
        let size = self.size();
        let knowledge = search_state.get_knowledge(self);
        let lookup = match knowledge.status {
            FindStatus::Winning(_) => Lookup::Winning,
            FindStatus::Losing { depth: old_depth } if old_depth >= depth => Lookup::LosingDeep,
            FindStatus::Losing { .. } => Lookup::LosingShallow,
            FindStatus::InProgress => Lookup::InProgress,
            FindStatus::Unknown => Lookup::Unknown,
        };
        stats::lookup(caller, lookup, size);
        match lookup {
            Lookup::Winning => return FindResult::Winning,
            Lookup::LosingDeep | Lookup::InProgress => return FindResult::Losing,
            Lookup::LosingShallow | Lookup::Unknown => {}
        }
        // States at the size limit can only be reduced, so re-evaluating one costs just the
        // lookups of its two (stored) reductions. Without `store_at_limit` only their wins
        // are stored (by `check_reductions`, needed for printing the strategy). Skipping
        // `InProgress` for them is safe: every cycle contains an insert move, which
        // decreases `depth`.
        if size >= max_size && !search_state.store_at_limit {
            stats::inc(Counter::unstored_at_limit, size);
            if depth == 0 {
                stats::inc(Counter::depth_zero, size);
                return FindResult::Losing;
            }
            let result = self.check_reductions(search_state, depth, max_size);
            if let FindResult::Losing = result {
                stats::inc(Counter::at_limit_lose, size);
            }
            return result;
        }
        if depth == 0 {
            stats::inc(Counter::depth_zero, size);
            search_state.update_status(&self, BarrieredKnowledge::new_losing(self, 0));
            return FindResult::Losing;
        }

        search_state.update_status(&self, BarrieredKnowledge::new_in_progress(self));

        let reduction_search_depth = if self.size() >= max_size { depth } else { 1 };

        if let w @ FindResult::Winning =
            self.check_reductions(search_state, reduction_search_depth, max_size)
        {
            return w;
        }

        if self.size() >= max_size {
            stats::inc(Counter::at_limit_lose, size);
            search_state.update_status(&self, BarrieredKnowledge::new_losing(&self, depth));
            return FindResult::Losing;
        }

        let moves = self.moves().collect::<Vec<_>>();
        stats::inc(Counter::expansion, size);
        stats::add(Counter::moves_generated, size, moves.len() as u64);
        // moves.sort_by_key(|sm| sm.preferable_order());

        // let winning = moves.iter().find_map(
        let winning = moves.par_iter().find_map_any(|move_| {
            if let FindResult::Winning = move_.find_strategy(search_state, depth, max_size) {
                Some(move_.move_)
            } else {
                None
            }
        });

        if let Some(move_) = winning {
            stats::inc(Counter::expansion_win, size);
            search_state.update_status(
                self,
                BarrieredKnowledge::new_winning(self, WinningMove::Move(move_)),
            );
            return FindResult::Winning;
        }
        stats::inc(Counter::expansion_lose, size);
        search_state.update_status(&self, BarrieredKnowledge::new_losing(&self, depth));
        FindResult::Losing
    }

    fn check_reductions(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
    ) -> FindResult {
        let size = self.size();
        stats::inc(Counter::reduction_check, size);
        if let Some(FindResult::Winning) = self.try_moved_limit_front().map(|c| {
            c.find_strategy_from(search_state, depth, max_size, Caller::ReductionFront)
        }) {
            stats::inc(Counter::reduction_win_front, size);
            search_state.update_status(
                &self,
                BarrieredKnowledge::new_winning(self, WinningMove::Reduction(Reduction::Front)),
            );
            return FindResult::Winning;
        }
        if let Some(FindResult::Winning) = self.try_moved_limit_back().map(|c| {
            c.find_strategy_from(search_state, depth, max_size, Caller::ReductionBack)
        }) {
            stats::inc(Counter::reduction_win_back, size);
            search_state.update_status(
                self,
                BarrieredKnowledge::new_winning(self, WinningMove::Reduction(Reduction::Back)),
            );
            return FindResult::Winning;
        }

        FindResult::Losing
    }
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
    ) -> FindResult {
        let size = self.state.size();
        stats::inc(Counter::move_tried, size);
        let mut cs = self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
            .collect::<Vec<_>>();
        cs.shuffle(&mut rand::rng());
        for color in cs {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            stats::inc(Counter::colour_branch, size);
            match clone.find_strategy_from(search_state, depth - 1, max_size, Caller::Move) {
                FindResult::Losing => {
                    stats::inc(Counter::move_refuted, size);
                    return FindResult::Losing;
                }
                FindResult::Winning => {}
            }
        }
        FindResult::Winning
    }
}

/// Tests that run a search hold this lock: the statistics counters are process-global, so
/// a concurrent search would leak into another test's counts.
#[cfg(test)]
static SEARCH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(all(test, feature = "stats"))]
mod stats_test {
    use super::*;
    use crate::stats::{lookup_counter, snapshot, Caller::*, Lookup::*};

    /// Runs a small search in both storage modes and checks the counters satisfy the
    /// relations the search code implies. A miscounted path, or an unexpected control-flow
    /// path, breaks one of them. (One test: the counters are global.)
    #[test]
    fn counters_are_consistent() {
        let _lock = SEARCH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        check_counters(true);
        check_counters(false);
    }

    fn check_counters(store_at_limit: bool) {
        let search_state = SearchState::<4>::new().with_store_at_limit(store_at_limit);
        let before = snapshot();
        let depths = 3..=9u8;
        for depth in depths.clone() {
            if let FindResult::Winning = State::<4>::new().find_strategy(&search_state, depth, 7) {
                break;
            }
        }
        let s = snapshot().since(&before);
        let t = |c| s.total(c);
        let searched = s.outcome(LosingShallow) + s.outcome(Unknown);
        println!("store_at_limit={store_at_limit}: {}", s.summary_line());

        assert!(t(Counter::expansion) > 1000, "search too small to be meaningful");
        assert!(s.lookups(Root) as usize <= depths.count());
        assert_eq!(t(Counter::colour_branch), s.lookups(Move));
        assert_eq!(searched, t(Counter::depth_zero) + t(Counter::reduction_check));
        assert_eq!(
            t(Counter::reduction_check),
            t(Counter::reduction_win_front)
                + t(Counter::reduction_win_back)
                + t(Counter::at_limit_lose)
                + t(Counter::expansion)
        );
        assert_eq!(
            t(Counter::expansion),
            t(Counter::expansion_win) + t(Counter::expansion_lose)
        );
        assert!(s.lookups(ReductionFront) <= t(Counter::reduction_check));
        assert!(s.lookups(ReductionBack) <= t(Counter::reduction_check));
        assert!(t(Counter::move_refuted) <= t(Counter::move_tried));
        assert!(t(Counter::move_tried) <= t(Counter::moves_generated));
        // Every searched state writes `InProgress` (or `Losing{0}` at depth 0), plus one
        // final status unless a reduction or expansion returned early with a win.
        // Unstored states at the limit write only their reduction wins.
        let unstored = t(Counter::unstored_at_limit);
        assert_eq!(store_at_limit, unstored == 0);
        assert_eq!(
            t(Counter::status_write),
            t(Counter::depth_zero)
                + t(Counter::reduction_check)
                + t(Counter::reduction_win_front)
                + t(Counter::reduction_win_back)
                + t(Counter::at_limit_lose)
                + t(Counter::expansion)
                - unstored
                - if store_at_limit { 0 } else { t(Counter::at_limit_lose) }
        );
        // Lookups made by the root only ever see a state searched at a lower depth.
        assert_eq!(t(lookup_counter(Root, Unknown)), 1);
        // Reduction children are mostly states already searched, so their lookups should
        // hit the map. A key mismatch between lookup and store (like the stray-bits bug in
        // `try_moved_limit_back`) shows up here: 42.9% back misses with the bug, ~0.1%
        // without.
        for caller in [ReductionFront, ReductionBack] {
            let misses = t(lookup_counter(caller, Unknown));
            assert!(
                misses * 10 < s.lookups(caller),
                "{caller:?}: {misses} of {} lookups missed the map",
                s.lookups(caller)
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Both storage modes must find a win for MQ=4, MS=8 and keep every winning state the
    /// strategy needs (`print_strategy` panics if one is missing). Validity of the printed
    /// strategy is checked by `cxx/strategy_check` (see the `bench` binary).
    #[test]
    fn both_storage_modes_find_a_printable_strategy() {
        let _lock = SEARCH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        for store_at_limit in [true, false] {
            let search_state = SearchState::<4>::new().with_store_at_limit(store_at_limit);
            let root = State::<4>::new();
            let won = (3..=30u8).any(|depth| {
                matches!(root.find_strategy(&search_state, depth, 8), FindResult::Winning)
            });
            assert!(won, "store_at_limit={store_at_limit}: no win found");
            let mut out = Vec::new();
            root.print_strategy(&search_state, &mut out);
            assert!(out.len() > 1000, "store_at_limit={store_at_limit}: strategy too short");
        }
    }
}
