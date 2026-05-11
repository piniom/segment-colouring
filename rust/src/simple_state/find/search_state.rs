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

#[derive(Debug, Clone, Default)]
pub struct RepresentativeKnowledge {
    barriered: Vec<BarrieredKnowledge>, 
}

#[derive(Debug, Default, Clone)]
pub struct SearchState<const MAX_CLIQUE: u32> {
    pub map: HashMap<Representative<MAX_CLIQUE>, RepresentativeKnowledge>,
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    pub fn get_knowledge(&self, state: &State<MAX_CLIQUE>) -> BarrieredKnowledge {
        self.get_applicable(state)
            .min_by_key(|sk| sk.status.success_key())
            .cloned()
            .unwrap_or_default()
            .combine_barrier(state)
    }
    pub fn update_status(&mut self, state: &State<MAX_CLIQUE>, knowledge: BarrieredKnowledge) {
        let state = state.set_barrier_as_limits(&state.representative_barrier(knowledge.barrier));
        let knowledge = BarrieredKnowledge::with_default_barrier(knowledge.status);
        for substate in state.substates() {
            self.update_status_inner(&substate, knowledge);
        }
    }
    fn update_status_inner(&mut self, state: &State<MAX_CLIQUE>, knowledge: BarrieredKnowledge) {
        let representative = Representative::new(*state);
        let barriered = knowledge.combine_barrier(state);
        let mut knowledge = self.map.remove(&representative).unwrap_or_default();
        knowledge.barriered = knowledge
            .barriered
            .into_iter()
            .filter(|sk| sk.should_not_be_overridden_by(&barriered))
            .collect();
        knowledge.barriered.push(barriered);
        self.map.insert(representative, knowledge);
    }

    fn get_applicable<'a>(
        &'a self,
        state: &'a State<MAX_CLIQUE>,
    ) -> impl Iterator<Item = &'a BarrieredKnowledge> {
        let representative = Representative::new(*state);
        self.map
            .get(&representative)
            .map(|k| &k.barriered)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .filter(|sk| sk.applies_to(state))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BarrieredKnowledge {
    pub barrier: FindBarrier,
    pub status: FindStatus,
}

impl BarrieredKnowledge {
    pub fn new_winning(barrier: FindBarrier, move_: WinningMove) -> Self {
        Self {
            barrier,
            status: FindStatus::Winning(move_),
        }
    }
    pub fn new_losing(depth: usize) -> Self {
        Self::with_default_barrier(FindStatus::Losing { depth })
    }
    pub fn new_in_progress() -> Self {
        Self::with_default_barrier(FindStatus::InProgress)
    }
    pub fn with_default_barrier(status: FindStatus) -> Self {
        Self {
            barrier: FindBarrier::default(),
            status,
        }
    }
    pub fn applies_to<const MAX_CLIQUE: u32>(&self, state: &State<MAX_CLIQUE>) -> bool {
        state.limits_to_barrier() >= self.barrier
    }
    fn should_not_be_overridden_by(&self, other: &BarrieredKnowledge) -> bool {
        !self.might_be_overridden_by(other)
            && self.status.success_key() < other.status.success_key()
    }
    fn might_be_overridden_by(&self, other: &BarrieredKnowledge) -> bool {
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
pub enum FindStatus {
    Winning(WinningMove),
    Losing {
        depth: usize,
    },
    InProgress,
    #[default]
    Unknown,
}

impl FindStatus {
    fn success_key(&self) -> usize {
        match self {
            FindStatus::Winning(_) => 0,
            FindStatus::InProgress => 1,
            FindStatus::Losing { depth } => usize::MAX - 1 - *depth,
            FindStatus::Unknown => usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WinningMove {
    Move(Move),
    Reduction(Reduction),
}
