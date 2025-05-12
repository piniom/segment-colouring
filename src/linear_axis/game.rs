use std::{
    collections::{HashMap, HashSet},
    isize,
};

use serde::de;

use crate::linear_axis::LinearAxis;

use super::{
    clicqued::ClicquedLinearAxis,
    normalization::NormalizedState,
    strategy::{StrategyConsumer, StrategyMove, StrategyState},
    History,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateStatus {
    True(Option<StrategyMove>),
    False(isize),
    Active,
}

impl StateStatus {
    pub fn to_result(&self) -> isize {
        match self {
            StateStatus::True(_) => isize::MAX,
            StateStatus::False(result) => *result,
            Self::Active => isize::MIN,
        }
    }
}
#[derive()]
pub struct Game {
    axis: ClicquedLinearAxis,
    history: Vec<History>,
    force_num_colours: usize,
    max_events: usize,
    states: HashMap<NormalizedState, StateStatus>,
    #[allow(dead_code)]
    reductees: HashMap<NormalizedState, HashSet<(NormalizedState, History)>>,
    pub strategy: StrategyConsumer,
}

impl Game {
    pub fn new(
        max_events: usize,
        max_clicque: usize,
        force_num_colours: usize,
        strategy: StrategyConsumer,
    ) -> Self {
        Self {
            axis: ClicquedLinearAxis::new(max_clicque),
            history: vec![],
            force_num_colours,
            max_events,
            states: HashMap::new(),
            reductees: HashMap::new(),
            strategy,
        }
    }
    pub fn simulate(&mut self, depth: isize) -> bool {
        let result = self.simulate_inner(depth, isize::MIN, isize::MAX);
        dbg!(result);
        let result = result == isize::MAX;
        if result {
            self.walk_strategy(&mut HashSet::new());
        }
        result
    }
    fn heuristic(&self) -> isize {
        self.axis.colours_used() as isize
            / self.axis.intersections.iter().copied().sum::<usize>() as isize
    }
    fn simulate_inner(&mut self, depth: isize, mut alpha: isize, mut beta: isize) -> isize {
        let normalized = self.normalize();
        if let Some(status) = self.get_state(&normalized) {
            return status.to_result();
        }
        if self.axis.colours_used() >= self.force_num_colours {
            self.report_success(None);
            return isize::MAX;
        }
        if self.check_reductions() {
            return isize::MAX;
        }
        if depth == 0 {
            return self.heuristic();
        }

        self.states.insert(normalized.clone(), StateStatus::Active);

        if self.axis.inner.events.len() >= self.max_events {
            return self.force_reductions(depth, alpha, beta);
        }
        let mut max = isize::MIN;
        for (s, e) in self.axis.valid_new_segments() {
            let mut min = isize::MAX;
            for c in self.uncollisions(s, e) {
                let mov = History::SegmentInsert {
                    start_index: s,
                    end_index: e,
                    color: c,
                };
                let reverse = self.apply_history(mov).unwrap();
                let result = self.simulate_inner(depth - 1, alpha, beta);
                self.apply_history(reverse);
                min = min.min(result);
                beta = beta.min(min);
            }
            max = max.max(min);
            alpha = alpha.max(max);
            if max >= self.force_num_colours as isize {
                self.report_success(Some(StrategyMove::Insert { start: s, end: e }));
                return isize::MAX;
            }
            if beta >= alpha {
                // dbg!(beta, alpha, depth);
                break;
            }
        }
        self.states.insert(normalized, StateStatus::False(max));
        max
    }
    fn check_reductions(&mut self) -> bool {
        for reduction in [History::LimitFront, History::LimitBack] {
            let reverse = if let Some(r) = self.apply_history(reduction) {
                r
            } else {
                return false;
            };
            let normalized = self.normalize();
            self.apply_history(reverse);
            if let Some(StateStatus::True(_)) = self.get_state(&normalized) {
                self.report_success(Some(reduction.strategy_move().unwrap()));
                return true;
            }
        }
        false
    }
    fn force_reductions(&mut self, depth: isize, alpha: isize, beta: isize) -> isize {
        let mut max = self.axis.colours_used() as isize;
        for reduction in [History::LimitFront, History::LimitBack] {
            let reverse = self.apply_history(reduction).unwrap();
            let result = self.simulate_inner(depth - 1, alpha, beta);
            self.apply_history(reverse);
            if result >= self.force_num_colours as isize {
                self.report_success(Some(reduction.strategy_move().unwrap()));
                return result;
            }
            max = result.max(max);
        }
        max
    }
    fn report_success(&mut self, mv: Option<StrategyMove>) {
        let normalized = self.normalize();
        if let Some(StateStatus::True(_)) = self.get_state(&normalized) {
            return;
        }
        self.states.insert(normalized, StateStatus::True(mv));
    }
    fn normalize(&self) -> NormalizedState {
        self.axis.strategy_normalize_without_symmetry()
    }
    pub fn number_of_states(&self) -> usize {
        self.states.len()
    }
    fn apply_history(&mut self, mv: History) -> Option<History> {
        let result = self.axis.apply_history(mv);
        match mv {
            History::LimitBack | History::LimitFront | History::SegmentInsert { .. } => {
                if result.is_some() {
                    self.history.push(mv);
                }
            }
            _ => {
                self.history.pop();
            }
        }

        result
    }
    fn get_state(&self, normalized: &NormalizedState) -> Option<&StateStatus> {
        self.states
            .get(&normalized)
            .or(self.states.get(&normalized.flipped(self.axis.max_colors())))
    }
    fn get_actual_normalised(&self) -> Option<NormalizedState> {
        let normalized = self.normalize();
        if self.states.contains_key(&normalized) {
            return Some(normalized.clone());
        }
        let flipped = normalized.flipped(self.axis.max_colors());
        if self.states.contains_key(&flipped) {
            return Some(flipped);
        }
        None
    }
    fn uncollisions(&self, start: usize, end: usize) -> Vec<u8> {
        self.axis.uncollisions(start, end)
    }
    fn walk_strategy(&mut self, walked: &mut HashSet<NormalizedState>) {
        let normalized = self.get_actual_normalised().unwrap();
        if walked.contains(&normalized)
            || walked.contains(&normalized.flipped(self.axis.max_colors()))
        {
            return;
        } else {
            walked.insert(normalized.clone());
        }
        let mut new_axis = ClicquedLinearAxis::with_inner(
            LinearAxis::from_strategy_state(self.strategy_state()),
            self.axis.max_clicque,
        );
        std::mem::swap(&mut new_axis, &mut self.axis);
        match self.get_state(&normalized).copied() {
            Some(StateStatus::True(None)) => {
                assert!(self.axis.colours_used() >= self.force_num_colours)
            }
            Some(StateStatus::True(Some(mv))) => match mv {
                limit @ (StrategyMove::LimitBack | StrategyMove::LimitFront) => {
                    self.strategy.consume(&normalized, limit);
                    let reverse = self.apply_history(limit.history().unwrap()).unwrap();
                    self.walk_strategy(walked);
                    self.apply_history(reverse);
                }
                insert @ StrategyMove::Insert { start, end } => {
                    self.strategy.consume(&normalized, insert);

                    for c in self.uncollisions(start, end) {
                        let reverse = self
                            .apply_history(History::SegmentInsert {
                                start_index: start,
                                end_index: end,
                                color: c,
                            })
                            .unwrap();
                        self.walk_strategy(walked);
                        self.apply_history(reverse);
                    }
                }
            },
            e => panic!("Should be true: {e:?}"),
        }
        std::mem::swap(&mut new_axis, &mut self.axis);
    }
    fn strategy_state(&self) -> StrategyState {
        StrategyState::from(
            &self.get_actual_normalised().unwrap(),
            self.axis.max_colors(),
        )
    }
}
