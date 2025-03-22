use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{
    clicqued::ClicquedLinearAxis,
    normalization::CompressedState,
    strategy::{StrategyConsumer, StrategyMove},
    History,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StateStatus {
    True,
    False,
    Active,
}

impl StateStatus {
    pub fn to_bool(&self) -> bool {
        match self {
            StateStatus::True => true,
            StateStatus::False => false,
            StateStatus::Active => false,
        }
    }
    pub fn from_bool(val: bool) -> Self {
        if val {
            StateStatus::True
        } else {
            StateStatus::False
        }
    }
}
#[derive(Debug)]
pub struct Game {
    axis: ClicquedLinearAxis,
    history: Vec<History>,
    force_num_colours: usize,
    max_events: usize,
    states: HashMap<Rc<CompressedState>, StateStatus>,
    reductees: HashMap<Rc<CompressedState>, HashSet<(Rc<CompressedState>, History)>>,
    state_bank: HashSet<Rc<CompressedState>>,
    pub strategy: StrategyConsumer,
}

impl Game {
    pub fn new(
        max_events: usize,
        max_clicque: u32,
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
            state_bank: HashSet::new(),
            strategy,
        }
    }
    pub fn number_of_states(&self) -> usize {
        self.states.len()
    }
    pub fn apply_history(&mut self, mv: History) -> Option<History> {
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
    pub fn simulate(&mut self) -> bool {
        let r = self.simulate_inner();
        println!("{:?}", self.states);
        r
    }
    fn simulate_inner(&mut self) -> bool {
        let normalized_state = self.get_from_bank(self.axis.normalize_compress());
        if self.axis.colours_used() >= self.force_num_colours {
            // println!("{}", self.axis.strategy_string());
            self.propagate_reductions(&normalized_state);
            self.states
                .insert(normalized_state.clone(), StateStatus::True);
            return true;
        }

        if let Some(value) = self.states.get(&normalized_state) {
            return value.to_bool();
        }

        for reduction in [History::LimitFront, History::LimitBack] {
            let reverse = match self.apply_history(reduction) {
                Some(r) => r,
                None => continue,
            };
            let r_norm = self.get_from_bank(self.axis.normalize_compress());
            self.apply_history(reverse);

            match self.states.get(&r_norm) {
                Some(StateStatus::True) => {
                    self.strategy.consume(
                        &self.axis.strategy_normalize(),
                        reduction.strategy_move().unwrap(),
                    );
                    return true;
                }
                _ => {
                    self.reductees
                        .entry(r_norm)
                        .or_default()
                        .insert((normalized_state.clone(), reduction));
                }
            };
        }

        self.states
            .insert(normalized_state.clone(), StateStatus::Active);

        if self.axis.inner.events.len() + 2 > self.max_events {
            for reduction in [History::LimitFront, History::LimitBack] {
                let reverse = match self.apply_history(reduction) {
                    Some(r) => r,
                    None => return false,
                };
                let result = self.simulate_inner();
                self.apply_history(reverse);

                if result {
                    self.propagate_reductions(&normalized_state);
                    self.states
                        .insert(normalized_state.clone(), StateStatus::True);
                    self.strategy.consume(
                        &self.axis.strategy_normalize(),
                        reduction.strategy_move().unwrap(),
                    );
                    return true;
                }
            }
            self.states.insert(normalized_state, StateStatus::False);
            return false;
        }

        let moves: Vec<_> = self.axis.valid_new_segments().collect();
        let mut result = false;

        for (s, e) in moves {
            let collisions = self.axis.segment_will_collide_with_colours(s, e);

            let mut all = true;

            for c in collisions
                .iter()
                .enumerate()
                .filter_map(|(i, c)| if *c { None } else { Some(i as u8) })
            {
                let mv = History::SegmentInsert {
                    start_index: s,
                    end_index: e,
                    color: c,
                };
                let reverse = self.apply_history(mv).unwrap();

                let simulation_result = self.simulate_inner();
                self.apply_history(reverse);
                if simulation_result == false {
                    all = false;
                    break;
                }
            }

            if all {
                result = true;
                self.strategy.consume(
                    &self.axis.strategy_normalize(),
                    StrategyMove::Insert { start: s, end: e },
                );
                break;
            }
        }

        if result {
            self.propagate_reductions(&normalized_state);
        }

        self.states
            .insert(normalized_state, StateStatus::from_bool(result));
        result
    }

    fn get_from_bank(&mut self, state: CompressedState) -> Rc<CompressedState> {
        match self.state_bank.get(&state) {
            Some(s) => s.clone(),
            None => {
                let value = Rc::new(state);
                self.state_bank.insert(value.clone());
                value
            }
        }
    }

    fn propagate_reductions(&mut self, state: &CompressedState) {
        if state.len() == 1 {
            return;
        }
        if let Some(StateStatus::True) = self.states.get(state) {
            return;
        }
        self.states.insert(
            self.state_bank.get(state).unwrap().clone(),
            StateStatus::True,
        );
        let reductees = self.reductees.get(state);
        if let Some(reductees) = reductees {
            for (r, _mov) in reductees.clone() {
                self.propagate_reductions(&r);
                // self.strategy.consume(&decompress_to_strategy(self.axis.max_colours, &r), mov.strategy_move().unwrap());
            }
        }
    }
}
