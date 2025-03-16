use serde::{Deserialize, Serialize};

use super::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum History {
    SegmentInsert {
        start_index: usize,
        end_index: usize,
        color: u8,
    },
    SegmentRemove {
        start_index: usize,
        end_index: usize,
    },
    LimitFront,
    LimitBack,
    EventInsertFront(Event),
    EventInsertBack(Event),
}