use dashmap::DashMap;
use smallvec::SmallVec;

use crate::{
    moves::Move,
    state::{compressed::CompressedState, find_barrier::FindBarrier, State},
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

// 12 segments * 5 bits - 4 bits = 56 bits = 7 bytes
const COMPRESSED_BYTES: usize = 7;

pub type Key = CompressedState<COMPRESSED_BYTES>;

/// Entries of one key when it holds more than one (rare: ~0.3% of keys for MQ=5).
type Entries = SmallVec<[BarrieredKnowledge; 4]>;

/// The state map. Most keys hold a single `BarrieredKnowledge`, stored packed into the
/// 32-bit value of `map` (12 bytes per slot). Keys with several entries keep them all in
/// `overflow`, and their `map` value is the `OVERFLOW` marker.
///
/// Locking: a writer holds the key's `map` entry (shard write lock) while it touches
/// `overflow`; a reader holds its `map` guard while reading `overflow`. Locks are always
/// taken `map` first, then `overflow`, so threads cannot deadlock, and readers always see
/// the slot and its overflow entries consistently.
#[derive(Debug)]
pub struct SearchState<const MAX_CLIQUE: u32> {
    pub map: DashMap<Key, PackedKnowledge>,
    overflow: DashMap<Key, Entries>,
    /// Whether states at the size limit are stored when losing (and marked `InProgress`).
    /// When `false` (the default), only winning ones are stored; the rest are re-evaluated
    /// from their (stored) reductions each time. They are the large majority of states.
    pub store_at_limit: bool,
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    /// A map with room for about `capacity` states, so it never has to grow (and leave
    /// freed old tables behind) while it fills up to that size.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: DashMap::with_capacity_and_shard_amount(capacity, 2usize.pow(15)),
            overflow: DashMap::with_shard_amount(2usize.pow(10)),
            store_at_limit: false,
        }
    }
    pub fn with_store_at_limit(mut self, store_at_limit: bool) -> Self {
        self.store_at_limit = store_at_limit;
        self
    }
}

/// Number of `entries_per_key` buckets; keys with more entries land in the last one.
pub const ENTRIES_BUCKETS: usize = 8;

#[derive(Debug, Clone, Copy, Default)]
pub struct CountStats {
    pub winning_states: isize,
    pub winning_substates: isize,
    pub total_states: isize,
    pub total_substates: isize,
    /// Only filled with `--features stats`: keys by number of stored entries.
    pub entries_per_key: [isize; ENTRIES_BUCKETS],
    /// Only filled with `--features stats`: keys by state size (number of segments).
    pub states_by_size: [isize; crate::stats::SIZE_BUCKETS],
}

impl<const MAX_CLIQUE: u32> SearchState<MAX_CLIQUE> {
    /// The most useful stored knowledge that applies to `state` (a normalised state).
    pub fn get_knowledge(&self, state: &State<MAX_CLIQUE>) -> BarrieredKnowledge {
        let key = CompressedState::new(*state);
        let Some(slot) = self.map.get(&key) else {
            return BarrieredKnowledge::default();
        };
        let best = if slot.is_overflow() {
            // `slot` stays locked while reading `overflow` (see `SearchState`).
            self.overflow.get(&key).and_then(|entries| {
                entries
                    .iter()
                    .filter(|k| k.applies_to(state))
                    .min_by_key(|k| k.status.success_key())
                    .copied()
            })
        } else {
            slot.unpack().filter(|k| k.applies_to(state))
        };
        best.unwrap_or_default()
    }

    /// All entries stored for a key (for debugging and statistics).
    pub fn entries(&self, key: &Key) -> Vec<BarrieredKnowledge> {
        match self.map.get(key) {
            Some(slot) => self.entries_of(key, *slot).into_vec(),
            None => Vec::new(),
        }
    }

    /// Entries of a key whose `map` value is `slot`; the caller holds the `map` guard.
    fn entries_of(&self, key: &Key, slot: PackedKnowledge) -> Entries {
        if slot.is_overflow() {
            self.overflow.get(key).map(|e| e.clone()).unwrap_or_default()
        } else {
            slot.unpack().into_iter().collect()
        }
    }

    pub fn update_status(&self, state: &State<MAX_CLIQUE>, knowledge: BarrieredKnowledge) {
        crate::stats::inc(crate::stats::Counter::status_write, state.size());
        let knowledge = BarrieredKnowledge::with_default_barrier(knowledge.status);
        self.update_status_inner(state, knowledge);
    }
    fn update_status_inner(&self, state: &State<MAX_CLIQUE>, mut knowledge: BarrieredKnowledge) {
        let mut state = *state;
        if state.normalize() {
            knowledge.barrier = knowledge.barrier.flip();
            if let FindStatus::Winning(move_) = &mut knowledge.status {
                move_.flip(state);
            }
        }
        let key = CompressedState::new(state);
        let barriered = knowledge.combine_barrier(&state);
        // Held until the end, so `overflow` is only changed under this key's lock.
        let mut slot = self.map.entry(key).or_insert(PackedKnowledge::EMPTY);
        let was_overflow = slot.is_overflow();
        let mut entries = self.entries_of(&key, *slot);

        entries.retain(|sk| !sk.might_be_overridden_by(&barriered));
        if !entries.iter().any(|sk| barriered.adds_nothing_given(sk)) {
            entries.push(barriered);
        }

        if state.is_symmetric() {
            let mut barriered = barriered;
            barriered.barrier = barriered.barrier.flip();
            if let FindStatus::Winning(move_) = &mut barriered.status {
                move_.flip(state);
            }
            if !entries.iter().any(|sk| barriered.adds_nothing_given(sk)) {
                entries.push(barriered);
            }
        }

        match entries.len() {
            0 | 1 => {
                if was_overflow {
                    self.overflow.remove(&key);
                }
                *slot = entries
                    .first()
                    .map_or(PackedKnowledge::EMPTY, PackedKnowledge::pack);
            }
            _ => {
                self.overflow.insert(key, entries);
                *slot = PackedKnowledge::OVERFLOW;
            }
        }
    }

    pub fn count_stats(&self) -> CountStats {
        let mut stats = CountStats::default();
        for item in self.map.iter() {
            let entries = self.entries_of(item.key(), *item.value());
            let winning = entries.iter().filter(|b| b.status.is_winning()).count();
            stats.winning_states += (winning > 0) as isize;
            stats.winning_substates += winning as isize;
            stats.total_states += 1;
            stats.total_substates += entries.len() as isize;
            if crate::stats::ENABLED {
                stats.entries_per_key[entries.len().min(ENTRIES_BUCKETS - 1)] += 1;
                // The key omits the implicit first event, so it has `len - 1` events.
                let size = (item.key().len() as usize + 1) / 2;
                stats.states_by_size[size.min(crate::stats::SIZE_BUCKETS - 1)] += 1;
            }
        }
        stats
    }
}

/// One `BarrieredKnowledge` packed into 32 bits (23 used), or a marker.
///
/// Layout: bits 0–2 tag, 3–7 barrier front, 8–12 barrier back, 13–22 payload:
/// - `WIN_MOVE`: move start (5 bits) and end (5 bits),
/// - `WIN_REDUCTION`: 0 = front, 1 = back,
/// - `LOSING`: depth (8 bits),
/// - `IN_PROGRESS`, `EMPTY`, `OVERFLOW`: no payload.
///
/// Barriers and move positions are event indexes, at most 28, so 5 bits each suffice.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct PackedKnowledge(u32);

impl PackedKnowledge {
    const TAG_EMPTY: u32 = 0;
    const TAG_WIN_MOVE: u32 = 1;
    const TAG_WIN_REDUCTION: u32 = 2;
    const TAG_LOSING: u32 = 3;
    const TAG_IN_PROGRESS: u32 = 4;
    const TAG_OVERFLOW: u32 = 7;

    /// No knowledge.
    pub const EMPTY: Self = Self(Self::TAG_EMPTY);
    /// The key's entries are in `SearchState::overflow`.
    pub const OVERFLOW: Self = Self(Self::TAG_OVERFLOW);

    fn tag(self) -> u32 {
        self.0 & 0b111
    }

    pub fn is_overflow(self) -> bool {
        self.tag() == Self::TAG_OVERFLOW
    }

    pub fn pack(knowledge: &BarrieredKnowledge) -> Self {
        let five = |v: u8| {
            debug_assert!(v < 32, "value {v} does not fit in 5 bits");
            (v & 0b1_1111) as u32
        };
        let (tag, payload) = match knowledge.status {
            FindStatus::Winning(WinningMove::Move(Move(start, end))) => {
                (Self::TAG_WIN_MOVE, five(start) | five(end) << 5)
            }
            FindStatus::Winning(WinningMove::Reduction(r)) => {
                (Self::TAG_WIN_REDUCTION, matches!(r, Reduction::Back) as u32)
            }
            FindStatus::Losing { depth } => (Self::TAG_LOSING, depth as u32),
            FindStatus::InProgress => (Self::TAG_IN_PROGRESS, 0),
            FindStatus::Unknown => unreachable!("Unknown is never stored"),
        };
        Self(tag | five(knowledge.barrier.front) << 3 | five(knowledge.barrier.back) << 8 | payload << 13)
    }

    /// The packed knowledge, or `None` for `EMPTY` (and `OVERFLOW`, which has no entry).
    pub fn unpack(self) -> Option<BarrieredKnowledge> {
        let field = |shift: u32, bits: u32| ((self.0 >> shift) & ((1 << bits) - 1)) as u8;
        let payload = self.0 >> 13;
        let status = match self.tag() {
            Self::TAG_WIN_MOVE => FindStatus::Winning(WinningMove::Move(Move(
                field(13, 5),
                field(18, 5),
            ))),
            Self::TAG_WIN_REDUCTION => FindStatus::Winning(WinningMove::Reduction(
                if payload & 1 == 1 {
                    Reduction::Back
                } else {
                    Reduction::Front
                },
            )),
            Self::TAG_LOSING => FindStatus::Losing {
                depth: field(13, 8),
            },
            Self::TAG_IN_PROGRESS => FindStatus::InProgress,
            _ => return None,
        };
        Some(BarrieredKnowledge {
            barrier: FindBarrier {
                front: field(3, 5),
                back: field(8, 5),
            },
            status,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BarrieredKnowledge {
    pub barrier: FindBarrier,
    pub status: FindStatus,
}

impl BarrieredKnowledge {
    pub fn new_winning<const MAX_CLIQUE: u32>(
        state: &State<MAX_CLIQUE>,
        move_: WinningMove,
    ) -> Self {
        Self {
            barrier: state.limits_to_barrier(),
            status: FindStatus::Winning(move_),
        }
    }
    pub fn new_losing<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>, depth: u8) -> Self {
        Self {
            barrier: state.limits_to_barrier(),
            status: FindStatus::Losing { depth },
        }
    }
    pub fn new_in_progress<const MAX_CLIQUE: u32>(state: &State<MAX_CLIQUE>) -> Self {
        Self {
            barrier: state.limits_to_barrier(),
            status: FindStatus::InProgress,
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
    /// Whether storing `self` would add nothing, given the stored `other`.
    ///
    /// Besides `might_be_overridden_by`, a win is redundant when another win already applies
    /// to every state it applies to: a win applies to states at most as confined as its
    /// barrier, so that is a win whose barrier is at least as confined (`<=`). Without this,
    /// re-evaluating a winning state (as happens to states at the size limit, which skip
    /// the lookup) stored the same win again on every visit.
    fn adds_nothing_given(&self, other: &BarrieredKnowledge) -> bool {
        self.might_be_overridden_by(other)
            || (self.status.is_winning()
                && other.status.is_winning()
                && other.barrier <= self.barrier)
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
        depth: u8,
    },
    InProgress,
    #[default]
    Unknown,
}

impl FindStatus {
    fn success_key(&self) -> u8 {
        match self {
            FindStatus::Winning(_) => 0,
            FindStatus::Losing { depth } => u8::MAX - 1 - *depth,
            FindStatus::InProgress => u8::MAX - 1,
            FindStatus::Unknown => u8::MAX,
        }
    }
    fn is_winning(&self) -> bool {
        match self {
            FindStatus::Winning(_) => true,
            _ => false,
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

#[cfg(test)]
mod test {
    use super::*;

    fn knowledge(front: u8, back: u8, status: FindStatus) -> BarrieredKnowledge {
        BarrieredKnowledge {
            barrier: FindBarrier { front, back },
            status,
        }
    }

    fn same(a: &BarrieredKnowledge, b: &BarrieredKnowledge) -> bool {
        format!("{a:?}") == format!("{b:?}")
    }

    /// Storing a win that is already stored (as happens when a state at the size limit is
    /// re-evaluated) must not add another entry.
    #[test]
    fn repeated_wins_are_not_duplicated() {
        let search_state = SearchState::<4>::new();
        let mut state = State::<4>::from_string("[ABab]");
        state.normalize();
        let win = knowledge(0, 0, FindStatus::Winning(WinningMove::Reduction(Reduction::Front)));
        for _ in 0..5 {
            search_state.update_status(&state, win);
        }
        let entries = search_state.entries(&CompressedState::new(state));
        // `[ABab]` is symmetric. Its mirrored win has the same barrier `(0, 0)`, so it
        // applies to exactly the same states and is redundant too.
        assert!(state.is_symmetric());
        assert_eq!(entries.len(), 1, "{entries:?}");

        // With an asymmetric barrier the mirrored win applies elsewhere, so both are kept,
        // still only once each however often the win is stored.
        let symmetric = State::<4>::from_string("A[Bab]");
        let mut norm = symmetric;
        norm.normalize();
        assert!(norm.is_symmetric());
        let fresh = SearchState::<4>::new();
        for _ in 0..5 {
            fresh.update_status(&norm, win);
        }
        let entries = fresh.entries(&CompressedState::new(norm));
        assert_eq!(entries.len(), 2, "{entries:?}");

        // An asymmetric state keeps a single entry.
        let mut state = State::<4>::from_string("[ABabCc]");
        state.normalize();
        assert!(!state.is_symmetric());
        for _ in 0..5 {
            search_state.update_status(&state, win);
        }
        let entries = search_state.entries(&CompressedState::new(state));
        assert_eq!(entries.len(), 1, "{entries:?}");

        // A win found with less room (more confined barrier) also applies with more room,
        // so a later win with more room adds nothing.
        let text = state.to_string().replacen('[', "", 1);
        let confined = State::<4>::from_string(&format!("{}[{}", &text[..1], &text[1..]));
        assert_eq!(confined.limit_front(), 1);
        let fresh = SearchState::<4>::new();
        fresh.update_status(&confined, win);
        for _ in 0..3 {
            fresh.update_status(&state, win);
        }
        let entries = fresh.entries(&CompressedState::new(state));
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0].barrier.front, 1, "{entries:?}");
    }

    #[test]
    fn slot_is_12_bytes() {
        assert_eq!(std::mem::size_of::<(Key, PackedKnowledge)>(), 12);
    }

    #[test]
    fn pack_round_trips() {
        let statuses = [
            FindStatus::Winning(WinningMove::Move(Move(0, 0))),
            FindStatus::Winning(WinningMove::Move(Move(28, 31))),
            FindStatus::Winning(WinningMove::Move(Move(7, 19))),
            FindStatus::Winning(WinningMove::Reduction(Reduction::Front)),
            FindStatus::Winning(WinningMove::Reduction(Reduction::Back)),
            FindStatus::Losing { depth: 0 },
            FindStatus::Losing { depth: 255 },
            FindStatus::Losing { depth: 37 },
            FindStatus::InProgress,
        ];
        for status in statuses {
            for (front, back) in [(0, 0), (31, 31), (3, 28), (28, 0)] {
                let k = knowledge(front, back, status);
                let packed = PackedKnowledge::pack(&k);
                assert!(!packed.is_overflow());
                assert_ne!(packed, PackedKnowledge::EMPTY);
                let unpacked = packed.unpack().unwrap();
                assert!(same(&k, &unpacked), "{k:?} -> {packed:?} -> {unpacked:?}");
            }
        }
        assert!(PackedKnowledge::EMPTY.unpack().is_none());
        assert!(PackedKnowledge::OVERFLOW.unpack().is_none());
        assert!(PackedKnowledge::OVERFLOW.is_overflow());
    }

    /// Many threads update and read the same few keys, with barriers that make keys hold
    /// several entries and move between the inline slot and `overflow`. Afterwards every
    /// key must be consistent (marker <=> overflow entry), and lookups must keep working.
    #[test]
    fn concurrent_updates_stay_consistent() {
        use rayon::prelude::*;
        // Writes go through the global statistics counters, like a search does.
        let _lock = super::super::SEARCH_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let search_state = SearchState::<4>::new();
        let states: Vec<State<4>> = ["[AaBb]", "A[aBb]", "[AaB]b", "[ABab]", "A[BaCb]c"]
            .iter()
            .map(|s| State::<4>::from_string(s))
            .collect();
        (0..20_000u32).into_par_iter().for_each(|i| {
            let state = states[i as usize % states.len()];
            let status = match i % 4 {
                0 => FindStatus::InProgress,
                1 => FindStatus::Losing { depth: (i % 13) as u8 },
                2 => FindStatus::Losing { depth: (i % 7) as u8 },
                _ => FindStatus::Winning(WinningMove::Reduction(Reduction::Front)),
            };
            search_state.update_status(&state, knowledge(0, 0, status));
            let _ = search_state.get_knowledge(&state);
        });
        for item in search_state.map.iter() {
            let in_overflow = search_state.overflow.contains_key(item.key());
            assert_eq!(item.value().is_overflow(), in_overflow);
            if in_overflow {
                assert!(search_state.overflow.get(item.key()).unwrap().len() >= 2);
            }
        }
        assert_eq!(
            search_state.overflow.len(),
            search_state.map.iter().filter(|i| i.value().is_overflow()).count()
        );
        for state in &states {
            let mut norm = *state;
            norm.normalize();
            assert!(!search_state.entries(&CompressedState::new(norm)).is_empty());
        }
    }
}
