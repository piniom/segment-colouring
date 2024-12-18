use std::collections::{HashMap, HashSet};

use crate::{
    axis::{Axis, SegmentId}, first_fit::ColourId, state_equivalence::NormalizedState
};

#[derive(Debug, Clone)]
pub struct Game {
    segments: Vec<SegmentId>,
    axis: Axis,
    force_num_colours: usize,
    max_segments: usize,
    states: HashMap<NormalizedState, bool>,
    colouring: HashMap<SegmentId, ColourId>
}

impl Game {
    pub fn new(max_segments: usize, max_clicque: u32, force_num_colours: usize) -> Self {
        Self {
            segments: vec![],
            axis: Axis::new(max_clicque),
            force_num_colours,
            max_segments,
            states: HashMap::new(),
            colouring: HashMap::new()
        }
    }
    pub fn simulate(&mut self) -> bool {
        // dbg!(&self.coluring);
        assert!(self.segments.iter().all(|s| self.colouring.contains_key(s)));
        assert!(self.axis.events().iter().all(|e| self.colouring.contains_key(&e.segment_id())));
        let normalized_state = self.normalized_state();
        
        if let Some(value) = self.states.get(&normalized_state) {
            return *value;
        }
        
        if self.segments.len() > self.max_segments {
            self.states.insert(normalized_state, false);
            return false
        }

        let moves: Vec<_> = self.possible_moves().collect();
        let mut result = false;
        
        for (s, e) in moves {
            let segment_id = self.axis.insert_segment(s, e).unwrap();
            self.segments.push(segment_id);
            let colours = self.not_colliding_colours(self.axis.segment_collides_with(segment_id).unwrap());

            let mut all = true;

            // dbg!(segment_id);

            for c in colours {
                self.colouring.insert(segment_id, c);
            
                if self.segments.iter().map(|s| self.colouring.get(s).unwrap()).collect::<HashSet<_>>().len() >= self.force_num_colours {
                    // println!("{}", self.axis.to_string(&self.colouring));
                }
                

                if self.simulate() == false {
                    all = false;
                    break;
                } 
                self.colouring.remove(&segment_id);
            }

            self.segments.pop();
            self.axis.remove_segment(segment_id);

            if all {
                result = true;
                break
            }
        }
        self.states.insert(normalized_state, result);
        result
    }
    fn possible_moves<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + use<'a> {
        self.axis
            .valid_indexes()
            .map(|s| self.axis.possible_ends(s).map(move |e| (s, e)))
            .flatten()
    }
    fn normalized_state(&self) -> NormalizedState {
        self.axis.normalized_state(&self.colouring)
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
        candidates.iter().enumerate().filter(|(_, &e)| e == true).map(|(i, _)| i as u32).collect()
    }
}
