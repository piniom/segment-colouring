use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{clicqued::ClicquedLinearAxis, normalization::NormalizedState, History};

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Game {
    axis: ClicquedLinearAxis,
    history: Vec<History>,
    force_num_colours: usize,
    max_events: usize,
    states: HashMap<Rc<NormalizedState>, StateStatus>,
    reductees: HashMap<Rc<NormalizedState>, HashSet<Rc<NormalizedState>>>,
    state_bank: HashSet<Rc<NormalizedState>>,
}

impl Game {
    pub fn new(max_events: usize, max_clicque: u32, force_num_colours: usize) -> Self {
        Self {
            axis: ClicquedLinearAxis::new(max_clicque),
            history: vec![],
            force_num_colours,
            max_events,
            states: HashMap::new(),
            reductees: HashMap::new(),
            state_bank: HashSet::new(),
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
        let normalized_state = self.get_from_bank(self.axis.normalize());
        if self.axis.colours_used() >= self.force_num_colours {
            println!("{}", self.axis.inner.to_string());
            self.states
                .insert(normalized_state.clone(), StateStatus::True);
            // self.propagate_reductions(&normalized_state);
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
            let r_norm = self.get_from_bank(self.axis.normalize());
            self.apply_history(reverse);

            match self.states.get(&r_norm) {
                Some(StateStatus::True) => {
                    return true;
                }
                _ => {
                    self.reductees
                        .entry(r_norm)
                        .or_default()
                        .insert(normalized_state.clone());
                }
            };
        }

        self.states
            .insert(normalized_state.clone(), StateStatus::Active);

        if self.axis.inner.events.len() >= self.max_events {
            for reduction in [History::LimitFront, History::LimitBack] {
                let reverse = self.apply_history(reduction).unwrap();
                let result = self.simulate();
                self.apply_history(reverse);

                if result {
                    self.states
                        .insert(normalized_state.clone(), StateStatus::True);
                    self.propagate_reductions(&normalized_state);
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

                if *self.axis.intersections.iter().max().unwrap_or(&0) > self.axis.max_clicque {
                    dbg!(&collisions);
                    println!("{}", self.axis.inner.to_string());
                    println!("{:?}", self.axis.intersections);
                    self.apply_history(reverse);
                    println!("{}", self.axis.inner.to_string());
                    println!("{:?}", self.axis.intersections);
                    println!("{:?}", self.axis.inner.events);
                    dbg!(self.axis.valid_new_segment_ends(s));
                    dbg!(s, e);
                    dbg!(self.axis.segments_opened_at_front());
                    let mut ax = ClicquedLinearAxis::new(self.axis.max_clicque);
                    for mv in &self.history {
                        println!("{:?}", mv);
                        if ax.apply_history(*mv).is_none() {
                            dbg!(mv);
                            panic!();
                        }
                        println!("{}\n", ax.inner.to_string());
                    }
                    panic!();
                }

                let simulation_result = self.simulate();
                self.apply_history(reverse);
                if simulation_result == false {
                    all = false;
                    break;
                }
            }

            if all {
                result = true;
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

    fn get_from_bank(&mut self, state: NormalizedState) -> Rc<NormalizedState> {
        match self.state_bank.get(&state) {
            Some(s) => s.clone(),
            None => {
                let value = Rc::new(state);
                self.state_bank.insert(value.clone());
                value
            }
        }
    }

    fn propagate_reductions(&mut self, state: &NormalizedState) {
        if let Some(StateStatus::True) = self.states.get(state) {
            return;
        }
        let reductees = self.reductees.get(state);
        if let Some(reductees) = reductees {
            for r in reductees.clone() {
                self.propagate_reductions(&r);
                self.states.insert(r, StateStatus::True);
            }
        }
    }
}
