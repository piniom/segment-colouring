use crate::simple_state::{
    state::{find_barrier::FindBarrier, State},
    StateWithMove,
};

pub mod search_state;

use search_state::*;

#[derive(Debug, Clone, Copy)]
pub enum FindResult {
    Winning(FindBarrier),
    Losing,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy_root(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        for d in 2..=depth {
            println!("{d}");
            if let w @ FindResult::Winning(_) = self.find_strategy(search_state, d, max_size) {
                return w;
            }
        }
        return FindResult::Losing;
    }
    pub fn find_strategy(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let mut norm = *self;
        let was_flipped = norm.normalize();
        let result = norm.find_strategy_inner(search_state, depth, max_size);
        match result {
            FindResult::Winning(mut barrier) => {
                if was_flipped {
                    barrier = barrier.flip();
                };
                FindResult::Winning(barrier)
            }
            FindResult::Losing => FindResult::Losing,
        }
    }
    fn find_strategy_inner(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let knowledge = search_state.get_knowledge(self);
        // println!("{knowledge:?}");
        match knowledge.status {
            FindStatus::Winning(_) => return FindResult::Winning(knowledge.barrier),
            FindStatus::Losing { depth: old_depth } => {
                if old_depth >= depth {
                    return FindResult::Losing;
                }
            }
            FindStatus::InProgress => return FindResult::Losing,
            FindStatus::Unknown => {}
        }
        if depth == 0 {
            search_state.update_status(&self, BarrieredKnowledge::new_losing(self, 0));
            return FindResult::Losing;
        }

        search_state.update_status(&self, BarrieredKnowledge::new_in_progress(self));

        if let w @ FindResult::Winning(_) = self.check_reductions(search_state, 1, max_size)
        {
            return w;
        }

        if self.size() >= max_size {
            search_state.update_status(&self, BarrieredKnowledge::new_losing(&self, depth));
            return FindResult::Losing;
        }

        let mut moves = self.moves().collect::<Vec<_>>();
        moves.sort_by_key(|sm| sm.preferable_order());

        for move_ in moves {
            if let FindResult::Winning(barrier) = move_.find_strategy(search_state, depth, max_size)
            {
                search_state.update_status(
                    self,
                    BarrieredKnowledge::new_winning(self, WinningMove::Move(move_.move_)),
                );
                return FindResult::Winning(barrier);
            }
        }
        search_state.update_status(&self, BarrieredKnowledge::new_losing(&self, depth));
        FindResult::Losing
    }

    fn check_reductions(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        if let Some(FindResult::Winning(barrier)) = self
            .try_moved_limit_front()
            .map(|c| c.find_strategy(search_state, depth, max_size))
        {
            let barrier = barrier.limit_front();
            search_state.update_status(
                &self,
                BarrieredKnowledge::new_winning(self, WinningMove::Reduction(Reduction::Front)),
            );
            return FindResult::Winning(barrier);
        }
        if let Some((_clone, FindResult::Winning(barrier))) = self
            .try_moved_limit_back()
            .map(|c| (c, c.find_strategy(search_state, depth, max_size)))
        {
            let barrier = barrier.limit_back();
            search_state.update_status(
                self,
                BarrieredKnowledge::new_winning(self, WinningMove::Reduction(Reduction::Back)),
            );
            return FindResult::Winning(barrier);
        }

        FindResult::Losing
    }
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &mut SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let mut barrier = self.find_barrier();
        for color in self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
        {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            match clone.find_strategy(search_state, depth - 1, max_size) {
                FindResult::Losing => return FindResult::Losing,
                FindResult::Winning(new_barrier) => barrier = barrier.confine(&new_barrier),
            }
        }
        return FindResult::Winning(barrier);
    }
    fn preferable_order(&self) -> (u8, i8) {
        let confining_factor =
            self.move_.0 - self.state.limit_front() + self.state.limit_back() - self.move_.1;
        (self.allowed_colours_count(), -(confining_factor as i8))
    }
    fn find_barrier(&self) -> FindBarrier {
        self.state.find_barrier(self.move_.0, self.move_.1)
    }
}
