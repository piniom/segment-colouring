use dashmap::DashMap;

use crate::simple_state::{
    state::{find_barrier::FindBarrier, representative::Representative, State},
    Move,
};

#[derive(Debug, Clone, Copy)]
pub enum Reduction {
    Front,
    Back,
}

impl Reduction {
    pub fn flip(&self) -> Self {
        match self {
            Reduction::Front => Reduction::Back,
            Reduction::Back => Reduction::Front,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepresentativeKnowledge {
    barriered: Vec<BarrieredKnowledge>,
}

#[derive(Debug, Default)]
pub struct SearchState<const MAX_CLIQUE: u32> {
    pub map: DashMap<Representative<MAX_CLIQUE>, RepresentativeKnowledge>,
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    pub fn get_knowledge(&self, state: &State<MAX_CLIQUE>) -> BarrieredKnowledge {
        self.get_applicable(state)
            .into_iter()
            .min_by_key(|sk| sk.status.success_key())
            .unwrap_or_default()
        // .combine_barrier(state)
    }
    pub fn update_status(&self, state: &State<MAX_CLIQUE>, knowledge: BarrieredKnowledge) {
        // println!("{state:?} {knowledge:?}");
        // let state = state.set_barrier_as_limits(&state.representative_barrier(knowledge.barrier));
        let knowledge = BarrieredKnowledge::with_default_barrier(knowledge.status);
        for substate in state.substates() {
            self.update_status_inner(&substate, knowledge)
        }
        // self.update_status_inner(state, knowledge);
    }
    fn update_status_inner(
        &self,
        state: &State<MAX_CLIQUE>,
        mut knowledge: BarrieredKnowledge,
    ) {
        let mut state = *state;
        if state.normalize() {
            knowledge.barrier = knowledge.barrier.flip();
            if let FindStatus::Winning(move_) = &mut knowledge.status {
                move_.flip(state);
            }
        }
        let representative = Representative::new(state);
        let barriered = knowledge.combine_barrier(&state);
        let mut entry = self.map.entry(representative).or_default();
        let knowledge = entry.value_mut();
        knowledge
            .barriered
            .retain(|sk| !sk.might_be_overridden_by(&barriered));

        if knowledge
            .barriered
            .iter()
            .find(|sk| barriered.might_be_overridden_by(sk))
            .is_none()
        {
            knowledge.barriered.push(barriered);
        }

        if state.is_symmetric() {
            let mut barriered = barriered;
            barriered.barrier = barriered.barrier.flip();
            if let FindStatus::Winning(move_) = &mut barriered.status {
                move_.flip(state);
            }
            knowledge.barriered.push(barriered);
        }
    }

    pub fn get_applicable(
        &self,
        state: &State<MAX_CLIQUE>,
    ) -> Vec<BarrieredKnowledge> {
        let representative = Representative::new(*state);
        self.map
            .get(&representative)
            .map(|k| {
                k.barriered
                    .iter()
                    .filter(|sk| sk.applies_to(state))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BarrieredKnowledge {
    pub barrier: FindBarrier,
    pub status: FindStatus,
}

impl BarrieredKnowledge {
    pub fn new_winning<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>, move_: WinningMove) -> Self {
        Self {
            barrier: state.limits_to_barrier(),
            status: FindStatus::Winning(move_),
        }
    }
    pub fn new_losing<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>, depth: usize) -> Self {
        Self {
            barrier: state.limits_to_barrier(),
            status: FindStatus::Losing { depth }
        }
    }
    pub fn new_in_progress<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>) -> Self {
        Self {
            barrier: state.limits_to_barrier(),
            status: FindStatus::InProgress
        }
    }
    pub fn with_default_barrier(status: FindStatus) -> Self {
        Self {
            barrier: FindBarrier::default(),
            status,
        }
    }
    pub fn applies_to<const MAX_CLIQUE: u32>(&self, state: &State<MAX_CLIQUE>) -> bool {
        if self.status.is_winning() {
            state.limits_to_barrier() >= self.barrier
        } else {
            state.limits_to_barrier() <= self.barrier
        }
    }
    fn might_be_overridden_by(&self, other: &BarrieredKnowledge) -> bool {
        if self.status.is_winning() {
             false
        } else if self.status.success_key() < other.status.success_key() {
            false
        } else if other.status.is_winning() {
            other.barrier <= self.barrier
        } else {
            other.barrier >= self.barrier
        }
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
            FindStatus::Losing { depth } => usize::MAX - 1 - *depth,
            FindStatus::InProgress => usize::MAX - 1,
            FindStatus::Unknown => usize::MAX,
        }
    }
    fn is_winning(&self) -> bool {
        match self {
            FindStatus::Winning(_) => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WinningMove {
    Move(Move),
    Reduction(Reduction),
}

impl WinningMove {
    pub fn flip<const MAX_CLIQUE: u32>(&mut self, state: State<MAX_CLIQUE>) {
        match self {
            WinningMove::Move(move_) => *move_ = state.flip_move(*move_),
            WinningMove::Reduction(reduction) => *reduction = reduction.flip(),
        }
    }
}
