use ahash::HashMap;

use crate::simple_state::{
    state::{find_barrier::FindBarrier, representative::Representative, State},
    Move,
};

#[derive(Debug, Clone, Copy)]
pub enum Reduction {
    Front,
    Back,
}

#[derive(Debug, Default, Clone)]
pub struct SearchState<const MAX_CLIQUE: u32> {
    pub map: HashMap<Representative<MAX_CLIQUE>, Vec<StateKnowledge>>,
    pub reductees: HashMap<State<MAX_CLIQUE>, (State<MAX_CLIQUE>, Reduction)>,
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    pub fn get_knowledge(&self, state: &State<MAX_CLIQUE>) -> StateKnowledge {
        self.get_applicable(state)
            .min_by_key(|sk| sk.status.success_key())
            .cloned()
            .unwrap_or_default()
            .combine_barrier(state)
    }
    pub fn update_status(&mut self, state: &State<MAX_CLIQUE>, knowledge: StateKnowledge) {
        let representative = Representative::new(*state);
        let knowledge = knowledge.combine_barrier(state);
        let mut not_overridden = self
            .map
            .remove(&representative)
            .unwrap_or_default()
            .into_iter()
            .filter(|sk| sk.should_not_be_overridden_by(&knowledge))
            .collect::<Vec<_>>();
        not_overridden.push(knowledge);
        self.map.insert(representative, not_overridden);
    }

    fn get_applicable<'a>(
        &'a self,
        state: &'a State<MAX_CLIQUE>,
    ) -> impl Iterator<Item = &'a StateKnowledge> {
        let representative = Representative::new(*state);
        self.map
            .get(&representative)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .filter(|sk| sk.applies_to(state))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StateKnowledge {
    pub barrier: FindBarrier,
    pub status: StateKnowledgeStatus,
}

impl StateKnowledge {
    pub fn new_winning(barrier: FindBarrier, move_: WinningMove) -> Self {
        Self {
            barrier,
            status: StateKnowledgeStatus::Winning(move_),
        }
    }
    pub fn new_losing(depth: usize) -> Self {
        Self::with_default_barrier(StateKnowledgeStatus::Losing { depth })
    }
    pub fn new_in_progress() -> Self {
        Self::with_default_barrier(StateKnowledgeStatus::InProgress)
    }
    pub fn with_default_barrier(status: StateKnowledgeStatus) -> Self {
        Self {
            barrier: FindBarrier::default(),
            status,
        }
    }
    pub fn applies_to<const MAX_CLIQUE: u32>(&self, state: &State<MAX_CLIQUE>) -> bool {
        state.limits_to_barrier() >= self.barrier
    }
    fn should_not_be_overridden_by(&self, other: &StateKnowledge) -> bool {
        !self.might_be_overridden_by(other)
            && self.status.success_key() < other.status.success_key()
    }
    fn might_be_overridden_by(&self, other: &StateKnowledge) -> bool {
        other.barrier <= self.barrier
    }
    pub fn combine_barrier<const MAX_CLIQUE: u32>(&self, state: &State<MAX_CLIQUE>) -> Self {
        Self {
            barrier: state.representative_barrier(self.barrier),
            status: self.status,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum StateKnowledgeStatus {
    Winning(WinningMove),
    Losing {
        depth: usize,
    },
    InProgress,
    #[default]
    Unknown,
}

impl StateKnowledgeStatus {
    fn success_key(&self) -> usize {
        match self {
            StateKnowledgeStatus::Winning(_) => 0,
            StateKnowledgeStatus::InProgress => 1,
            StateKnowledgeStatus::Losing { depth } => usize::MAX - 1 - *depth,
            StateKnowledgeStatus::Unknown => usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WinningMove {
    Move(Move),
    Reduction(Reduction),
}
