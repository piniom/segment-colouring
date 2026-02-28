use ahash::HashMap;

use crate::simple_state::state::State;

pub mod state;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move(u8, u8);

#[derive(Debug, Default)]
pub enum Visited {
    #[default]
    No,
    Active,
    Winning(Move),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateWithMove<'a, const MAX_CLIQUE: u32> {
    state: &'a State<MAX_CLIQUE>,
    move_: Move,
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn allowed_colours_count(&self) -> u8 {
        self.state
            .allowed_colours_for_segment_bits(self.move_.0, self.move_.1)
            .count_ones() as u8
    }
    pub fn outcomes(&'a self) -> impl Iterator<Item = State<MAX_CLIQUE>> + use<'a, MAX_CLIQUE> {
        self.state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
            .map(move |c| {
                let mut clone = self.state.clone();
                clone.insert_segment(self.move_.0, self.move_.1, c as u8);
                clone
            })
    }
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(&self, _map: &mut HashMap<Self, Visited>) {}
    pub fn moves<'a>(
        &'a self,
    ) -> impl Iterator<Item = StateWithMove<'a, MAX_CLIQUE>> + use<'a, MAX_CLIQUE> {
        (self.limit_front()..=self.limit_back()).flat_map(move |start| {
            let (a, b) = self.valid_segment_ends(start);
            (a..b).map(move |end| self.with_move(Move(start, end)))
        })
    }
    pub fn front_moves<'a>(
        &'a self,
    ) -> impl Iterator<Item = StateWithMove<'a, MAX_CLIQUE>> + use<'a, MAX_CLIQUE> {
        let (a, b) = self.valid_segment_ends(0);
        (a..b).map(move |end| self.with_move(Move(0, end)))
    }
    pub fn with_move<'a>(&'a self, move_: Move) -> StateWithMove<'a, MAX_CLIQUE> {
        StateWithMove { state: self, move_ }
    }
}
