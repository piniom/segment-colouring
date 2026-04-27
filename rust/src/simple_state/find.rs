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

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        map: &mut HashMap<Self, Visited>,
        depth: usize,
        max_size: u8,
    ) -> bool {
        if depth == 0 {
            return false;
        }
        match map.get(self).copied().unwrap_or_default() {
            Visited::Winning(_) => return true,
            Visited::Losing => return false,
            Visited::Active(count) => {
                if count == 0 {
                    return false;
                }
                map.insert(*self, Visited::Active(count - 1));
            }
            Visited::No => {
                map.insert(*self, Visited::Active(5));
            }
        }

        if self.size() == max_size {
            let cloned = *self;
            cloned.limit_front();
            if cloned.find_strategy(map, depth - 1, max_size) {
                return true;
            }
            let cloned = *self;
            cloned.limit_back();
            return cloned.find_strategy(map, depth - 1, max_size);
        }
        for move_ in self.moves() {
            if move_.find_strategy(map, depth, max_size) {
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
        max_size: u8,
    ) -> bool {
        // if self
        //     .state
        //     .allowed_colours_for_segment(self.move_.0, self.move_.1)
        //     .count()
        //     >= 6
        // {
        //     let mut norm = *self.state;
        //     norm.normalize();
        //     map.insert(norm, Visited::Losing);
        //     return false;
        // }
        for color in self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
        {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            if !(clone.find_strategy(map, depth - 1, max_size)) {
                let mut norm = *self.state;
                norm.normalize();
                map.insert(norm, Visited::Losing);
                return false;
            }
        }
        let mut norm = *self.state;
        let move_ = if norm.normalize() {
            norm.flip_move(self.move_)
        } else {
            self.move_
        };
        map.insert(norm, Visited::Winning(move_));
        return true;
    }
}
