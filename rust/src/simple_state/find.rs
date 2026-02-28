use ahash::HashMap;

use crate::simple_state::{Move, state::State};

#[derive(Debug, Default)]
pub enum Visited {
    #[default]
    No,
    Active,
    Winning(Move),
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(&self, _map: &mut HashMap<Self, Visited>) {}
}