use crate::simple_state::{state::State, StateWithMove};
use rayon::prelude::*;
pub mod root;

pub mod search_state;

use search_state::*;

#[derive(Debug, Clone, Copy)]
pub enum FindResult {
    Winning,
    Losing,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let mut norm = *self;
        norm.normalize();
        norm.find_strategy_inner(search_state, depth, max_size)
    }
    fn find_strategy_inner(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let knowledge = search_state.get_knowledge(self);
        match knowledge.status {
            FindStatus::Winning(_) => return FindResult::Winning,
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

        let reduction_search_depth = if self.size() >= max_size { depth } else { 1 };

        if let w @ FindResult::Winning =
            self.check_reductions(search_state, reduction_search_depth, max_size)
        {
            return w;
        }

        if self.size() >= max_size {
            search_state.update_status(&self, BarrieredKnowledge::new_losing(&self, depth));
            return FindResult::Losing;
        }

        let mut moves = self.moves().collect::<Vec<_>>();
        // moves.sort_by_key(|sm| sm.preferable_order());

        // let winning = moves.iter().find_map(
        let winning = moves.par_iter().find_map_any(
            |move_| {
            if let FindResult::Winning = move_.find_strategy(search_state, depth, max_size) {
                Some(move_.move_)
            } else {
                None
            }
        });

        if let Some(move_) = winning {
            search_state.update_status(
                self,
                BarrieredKnowledge::new_winning(self, WinningMove::Move(move_)),
            );
            return FindResult::Winning;
        }
        search_state.update_status(&self, BarrieredKnowledge::new_losing(&self, depth));
        FindResult::Losing
    }

    fn check_reductions(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        if let Some(FindResult::Winning) = self
            .try_moved_limit_front()
            .map(|c| c.find_strategy(search_state, depth, max_size))
        {
            search_state.update_status(
                &self,
                BarrieredKnowledge::new_winning(self, WinningMove::Reduction(Reduction::Front)),
            );
            return FindResult::Winning;
        }
        if let Some(FindResult::Winning) = self
            .try_moved_limit_back()
            .map(|c| c.find_strategy(search_state, depth, max_size))
        {
            search_state.update_status(
                self,
                BarrieredKnowledge::new_winning(self, WinningMove::Reduction(Reduction::Back)),
            );
            return FindResult::Winning;
        }

        FindResult::Losing
    }
}

impl<'a, const MAX_CLIQUE: u32> StateWithMove<'a, MAX_CLIQUE> {
    pub fn find_strategy(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        for color in self
            .state
            .allowed_colours_for_segment(self.move_.0, self.move_.1)
        {
            let mut clone = *self.state;
            clone.insert_segment(self.move_.0, self.move_.1, color);
            match clone.find_strategy(search_state, depth - 1, max_size) {
                FindResult::Losing => return FindResult::Losing,
                FindResult::Winning => {}
            }
        }
        FindResult::Winning
    }
    fn preferable_order(&self) -> (u8, i8) {
        let confining_factor =
            self.move_.0 - self.state.limit_front() + self.state.limit_back() - self.move_.1;
        (self.allowed_colours_count(), (confining_factor as i8))
    }
}
