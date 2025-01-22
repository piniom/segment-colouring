use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    axis::{Axis, SegmentId},
    first_fit::ColourId,
    state_equivalence::NormalizedState,
};

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
    segments: Vec<SegmentId>,
    axis: Axis,
    force_num_colours: usize,
    max_segments: usize,
    states: HashMap<Rc<NormalizedState>, StateStatus>,
    colouring: HashMap<SegmentId, ColourId>,
    reductees: HashMap<Rc<NormalizedState>, HashSet<Rc<NormalizedState>>>,
    state_bank: HashSet<Rc<NormalizedState>>,
}

impl Game {
    pub fn new(max_segments: usize, max_clicque: u32, force_num_colours: usize) -> Self {
        Self {
            segments: vec![],
            axis: Axis::new(max_clicque),
            force_num_colours,
            max_segments,
            states: HashMap::new(),
            colouring: HashMap::new(),
            reductees: HashMap::new(),
            state_bank: HashSet::new(),
        }
    }
    pub fn number_of_states(&self) -> usize {
        self.states.len()
    }
    pub fn simulate(&mut self) -> bool {
        let normalized_state = self.normalized_state();
        if self.colours_used() >= self.force_num_colours {
            self.states
                        .insert(normalized_state.clone(), StateStatus::True);
                    // self.propagate_reductions(&normalized_state);
                    return true;
        }

        if let Some(value) = self.states.get(&normalized_state) {
            return value.to_bool();
        }

        let current_reductions = self.current_reductions();

        for r in &current_reductions {
            let r_norm = self.get_from_bank(r.normalized_state(&self.colouring));
            match self.states.get(&r_norm) {
                Some(StateStatus::True) => return true,
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

        if self.axis.segments.len() >= self.max_segments {
            for a in current_reductions {
                let axis_clone = self.axis.clone();
                self.axis = a;
                let result = self.simulate();
                self.axis = axis_clone;
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

        let moves: Vec<_> = self.possible_moves().collect();
        let mut result = false;

        for &(s, e) in &moves {
            let axis_clone = self.axis.clone();
            let segment_id = self.insert_segment(s, e);
            if *self.axis.intersections.iter().max().unwrap_or(&0) > self.axis.max_clicque {
                println!("{}", axis_clone.to_string(&self.colouring));
                println!("{:?}", axis_clone.intersections);
                dbg!(s, e);
                println!("{:?}", moves);
                println!("{}", self.axis.to_string(&self.colouring));
                panic!();
            }
            
            let colours =
                self.not_colliding_colours(self.axis.segment_collides_with(segment_id).unwrap());

            let mut all = true;

            for c in colours {
                self.colouring.insert(segment_id, c);

                if self.simulate() == false {
                    all = false;
                    break;
                }

                self.colouring.remove(&segment_id);
            }

            self.segments.pop();
            self.axis = axis_clone;

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

    fn insert_segment(&mut self, start_index: usize, end_index: usize) -> SegmentId {
        let segment_id = self.axis.insert_segment(start_index, end_index).unwrap();
        self.segments.push(segment_id);
        segment_id
    }

    fn colour_segment(&mut self, segment_id: SegmentId, colour: ColourId) {
        self.colouring.insert(segment_id, colour);
    }

    pub fn insert_coloured_segment(&mut self, start_index: usize, end_index: usize, colour: ColourId) {
        let colour_id = self.insert_segment(start_index, end_index);
        self.colour_segment(colour_id, colour);
    }

    fn possible_moves<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + use<'a> {
        self.axis
            .valid_indexes()
            .map(|s| self.axis.possible_ends(s).map(move |e| (s, e)))
            .flatten()
            .filter(|(a, b)| a <= b)
    }

    fn normalized_state(&mut self) -> Rc<NormalizedState> {
        self.get_from_bank(self.axis.normalized_state(&self.colouring))
            .clone()
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

    fn current_reductions(&self) -> Vec<Axis> {
        let mut left = self.axis.clone();
        let mut right = self.axis.clone();
        let mut reductions = vec![];
        if left.confine_left() {
            reductions.push(left);
        }
        if right.confine_right() {
            reductions.push(right);
        }
        reductions
    }

    fn colliding_colours(
        &self,
        segments: impl IntoIterator<Item = SegmentId>,
    ) -> impl Iterator<Item = ColourId> {
        let set: HashSet<_> = segments
            .into_iter()
            .map(|s| *self.colouring.get(&s).unwrap())
            .collect();
        set.into_iter()
    }

    fn not_colliding_colours(
        &self,
        segments: impl IntoIterator<Item = SegmentId>,
    ) -> Vec<ColourId> {
        let mut candidates = vec![true; self.force_num_colours];
        for c in self.colliding_colours(segments) {
            candidates[c as usize] = false
        }
        candidates
            .iter()
            .enumerate()
            .filter(|(_, &e)| e == true)
            .map(|(i, _)| i as ColourId)
            .collect()
    }

    pub fn colours_used(&self) -> usize {
        self.segments.iter().map(|s| self.colouring.get(s).unwrap()).collect::<HashSet<_>>().len()
    }

    pub fn to_string(&self) -> String {
        self.axis.to_string(&self.colouring)
    }
}
