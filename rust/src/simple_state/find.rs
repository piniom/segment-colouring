use ahash::HashMap;

use crate::simple_state::{state::State, Move, StateWithMove};

#[derive(Debug, Default, Clone, Copy)]
pub enum Visited {
    #[default]
    No,
    Active(u8),
    Losing,
    Winning(Move),
}

#[derive(Debug, Clone, Copy)]
pub enum Reduction {
    Front,
    Back
}


#[derive(Debug, Default, Clone)]
pub struct SearchState<const MAX_CLIQUE: u32> {
    pub map: HashMap<State<MAX_CLIQUE>, Visited>,
    pub reductees: HashMap<State<MAX_CLIQUE>, (State<MAX_CLIQUE>, Reduction)>
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> bool {
        if depth == 0 {
            return false;
        }
        match search_state.map.get(self).copied().unwrap_or_default() {
            Visited::Winning(_) => return true,
            Visited::Losing => return false,
            Visited::Active(count) => {
                if count == 0 {
                    return false;
                }
                search_state.map.insert(*self, Visited::Active(count - 1));
            }
            Visited::No => {
                search_state.map.insert(*self, Visited::Active(0));
            }
        }

        if self.size() == max_size {
            let cloned = *self;
            cloned.limit_front();
            if cloned.find_strategy(search_state, depth - 1, max_size) {
                return true;
            }
            let cloned = *self;
            cloned.limit_back();
            return cloned.find_strategy(search_state, depth - 1, max_size);
        }

        let mut moves = self.moves().collect::<Vec<_>>();
        moves.sort_by_key(|sm| sm.allowed_colours_count());

        for move_ in moves {
            if move_.find_strategy(search_state, depth, max_size) {
                return true;
            }
        }

        false
    }
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> bool {
        for color in self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
        {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            if !(clone.find_strategy(search_state, depth - 1, max_size)) {
                let mut norm = *self.state;
                norm.normalize();
                // search_state.map.insert(norm, Visited::Losing);
                return false;
            }
        }
        let mut norm = *self.state;
        let move_ = if norm.normalize() {
            norm.flip_move(self.move_)
        } else {
            self.move_
        };
        search_state.map.insert(norm, Visited::Winning(move_));
        return true;
    }
    #[allow(dead_code)]
    fn allowed_colours(&self) -> usize {
        self.state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
            .count()
    }
}
