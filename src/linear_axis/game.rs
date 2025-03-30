use std::collections::{HashMap, HashSet};

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
    False,
    Active,
}

impl StateStatus {
    pub fn to_bool(&self) -> bool {
        match self {
            StateStatus::True(_) => true,
            StateStatus::False | StateStatus::Active => false,
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
    pub fn simulate(&mut self) -> bool {
        let result = self.simulate_inner();
        assert!(self.axis.inner.events.is_empty());
        assert!(self.history.is_empty());
        self.walk_strategy(&mut HashSet::new());
        result
    }
    fn simulate_inner(&mut self) -> bool {
        let normalized = self.normalize();
        if let Some(status) = self.get_state(&normalized) {
            return status.to_bool();
        }
        if self.axis.colours_used() >= self.force_num_colours {
            self.report_success(None);
            return true;
        }
        if self.check_reductions() {
            return true;
        }

        self.states.insert(normalized.clone(), StateStatus::Active);

        if self.axis.inner.events.len() >= self.max_events {
            return self.force_reductions();
        }

        for (s, e) in self.axis.valid_new_segments() {
            let mut all = true;
            for c in self.uncollisions(s, e) {
                let mov = History::SegmentInsert {
                    start_index: s,
                    end_index: e,
                    color: c,
                };
                let reverse = self.apply_history(mov).unwrap();
                let result = self.simulate_inner();
                self.apply_history(reverse);
                if !result {
                    all = false;
                    break;
                }
            }
            if all {
                self.report_success(Some(StrategyMove::Insert { start: s, end: e }));
                return true;
            }
        }
        self.states.insert(normalized, StateStatus::False);
        false
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
    fn force_reductions(&mut self) -> bool {
        for reduction in [History::LimitFront, History::LimitBack] {
            let reverse = self.apply_history(reduction).unwrap();
            let result = self.simulate_inner();
            self.apply_history(reverse);
            if result {
                self.report_success(Some(reduction.strategy_move().unwrap()));
                return true;
            }
        }
        false
    }
    fn report_success(&mut self, mv: Option<StrategyMove>) {
        let normalized = self.normalize();
        if self
            .get_state(&normalized)
            .map(StateStatus::to_bool)
            .unwrap_or(false)
        {
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
