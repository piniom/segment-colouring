use ahash::HashMap;

use crate::simple_state::{state::State, Move, StateWithMove};

#[derive(Debug, Default, Clone, Copy)]
pub enum Visited {
    #[default]
    No,
    Active,
    Losing,
    Winning(Move),
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(&self, map: &mut HashMap<Self, Visited>, depth: usize) -> bool {
        if depth == 0 {
            return false;
        }
        match map.get(self).copied().unwrap_or_default() {
            Visited::Winning(_) => return true,
            Visited::Losing => return false,
            Visited::Active => return false,
            Visited::No => (),
        }
        for move_ in self.moves() {
            if move_.find_strategy(map, depth) {
                return true;
            }
        }
        false
    }
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        map: &mut HashMap<State<MAX_CLIQUE>, Visited>,
        depth: usize,
    ) -> bool {
        for color in self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
        {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            if !(clone.find_strategy(map, depth - 1)) {
                let mut norm = *self.state;
                norm.normalize();
                map.insert(norm, Visited::Losing);
                return false;
            }
        }
        let mut norm = *self.state;
        norm.normalize();
        map.insert(norm, Visited::Winning(self.move_));
        return true;
    }
}
