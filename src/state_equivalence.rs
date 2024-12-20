use std::collections::HashMap;

use crate::{
    axis::{Axis, Event, SegmentId},
    first_fit::ColourId,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EventType {
    Start,
    End,
}

impl Event {
    pub fn ev_type(&self) -> EventType {
        match self {
            Event::Start(_) => EventType::Start,
            Event::End(_) => EventType::End,
        }
    }
}

pub type CompressedEvent = u8;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NormalizedState(pub(crate) Vec<CompressedEvent>);

impl NormalizedState {
    pub fn normalize(
        state: impl IntoIterator<Item = (EventType, SegmentId)>,
        colouring: &HashMap<SegmentId, ColourId>,
    ) -> Self {
        let state: Vec<_> = state.into_iter().collect();
        let mut normalized_colors: HashMap<ColourId, CompressedEvent> = HashMap::new();
        let mut normalized_state: Vec<CompressedEvent> = vec![];
        for (i, (e, c)) in state.into_iter().enumerate() {
            normalized_state.push(match e {
                EventType::Start => {
                    *normalized_colors
                        .entry(*colouring.get(&c).unwrap())
                        .or_insert(i as u8)
                        + 1
                }
                EventType::End => 0u8,
            });
        }
        NormalizedState(normalized_state)
    }
}

impl Axis {
    pub fn normalized_state(&self, colouring: &HashMap<SegmentId, ColourId>) -> NormalizedState {
        NormalizedState::normalize(
            self.events().iter().map(|e| (e.ev_type(), e.segment_id())),
            colouring,
        )
    }
}
