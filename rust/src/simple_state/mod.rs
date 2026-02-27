use ahash::HashMap;

use crate::simple_state::state::State;

pub mod state;

pub type Move = (u8, u8);

#[derive(Debug, Default)]
pub enum Visited {
    #[default]
    No,
    Active,
    Winning(Move)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateWithMove<const MAX_CLIQUE: u32>  {
    state: State<MAX_CLIQUE>,
    move_: Move
}

impl<const MAX_CLIQUE: u32> StateWithMove<MAX_CLIQUE>{
    pub fn allowed_colours_count(&self) -> u8 {
        self.state.allowed_colours_for_segment(self.move_.0, self.move_.1).count_ones() as u8
    }
    fn find_strategy(&self, map: &mut HashMap<Self, Visited>) {

    }
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE>{
    fn find_strategy(&self, map: &mut HashMap<Self, Visited>) {

    }
}