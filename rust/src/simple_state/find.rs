use ahash::HashMap;

use crate::simple_state::{
    state::{find_barrier::FindBarrier, State},
    Move, StateWithMove,
};

#[derive(Debug, Default, Clone, Copy)]
pub enum Visited {
    #[default]
    No,
    Active,
    Losing,
    Winning {
        move_: Move,
        barrier: FindBarrier,
    },
}

impl Visited {
    pub fn to_find_result(&self) -> FindStateResult {
        match *self {
            Self::Winning { barrier, .. } => FindStateResult::True(barrier),
            _ => FindStateResult::False,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Reduction {
    Front,
    Back,
}

#[derive(Debug, Default, Clone)]
pub struct SearchState<const MAX_CLIQUE: u32> {
    pub map: HashMap<State<MAX_CLIQUE>, Visited>,
    pub reductees: HashMap<State<MAX_CLIQUE>, (State<MAX_CLIQUE>, Reduction)>,
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    pub fn get_winning(&self, state: &State<MAX_CLIQUE>) -> Option<&Visited> {
        match self.map.get(state) {
            v @ Some(_) => v,
            None => {
                let mut flip = *state;
                flip.normalize_inner(false);
                self.map.get(&flip)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FindStateResult {
    True(FindBarrier),
    False,
}

#[derive(Debug, Clone, Copy)]
pub enum FindMoveResult {
    True(FindBarrier),
    False,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindStateResult {
        let mut norm = *self;
        let was_flipped = norm.normalize();
        let result = norm.find_strategy_inner(search_state, depth, max_size);
        match result {
            FindStateResult::True(mut barrier) => {
                if was_flipped {
                    barrier = barrier.flip();
                };
                FindStateResult::True(barrier)
            }
            FindStateResult::False => FindStateResult::False,
        }
    }
    fn find_strategy_inner(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindStateResult {
        if depth == 0 {
            return FindStateResult::False;
        }
        match search_state.map.get(&self).copied().unwrap_or_default() {
            Visited::Winning { barrier, .. } => return FindStateResult::True(barrier),
            Visited::Losing => return FindStateResult::False,
            Visited::Active => return FindStateResult::False,
            Visited::No => {
                search_state.map.insert(*self, Visited::Active);
            }
        }

        // if self.size() == max_size {
        //     let cloned = *self;
        //     cloned.limit_front();
        //     if cloned.find_strategy(search_state, depth - 1, max_size) {
        //         return true;
        //     }
        //     let cloned = *self;
        //     cloned.limit_back();
        //     return cloned.find_strategy(search_state, depth - 1, max_size);
        // }

        let mut moves = self.moves().collect::<Vec<_>>();
        moves.sort_by_key(|sm| sm.preferable_order());

        for move_ in moves {
            if let FindMoveResult::True(barrier) =
                move_.find_strategy(search_state, depth, max_size)
            {
                search_state.map.insert(
                    *self,
                    Visited::Winning {
                        move_: move_.move_,
                        barrier,
                    },
                );
                return FindStateResult::True(barrier);
            }
        }
        search_state.map.insert(*self, Visited::Losing);
        FindStateResult::False
    }
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindMoveResult {
        let mut barrier = self.find_barrier();
        for color in self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
        {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            match clone.find_strategy(search_state, depth - 1, max_size) {
                FindStateResult::False => return FindMoveResult::False,
                FindStateResult::True(new_barrier) => barrier = barrier.confine(&new_barrier),
            }
        }
        return FindMoveResult::True(barrier);
    }
    fn preferable_order(&self) -> (u8, u8) {
        let confining_factor = self.move_.0 - self.state.limit_front() + self.state.limit_back()
            - self.state.limit_back();
        (self.allowed_colours_count(), confining_factor)
    }
    fn find_barrier(&self) -> FindBarrier {
        self.state.find_barrier(self.move_.0, self.move_.1)
    }
}

#[test]
fn test() {
    let mut s = State::<3>::from_string("[AaABCacb]");
    s.flip();
    s.normalize_inner(false);
    panic!("{}", s.to_string())
}
