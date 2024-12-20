use super::{Segment, SegmentId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Start(SegmentId),
    End(SegmentId),
}

impl Event {
    pub fn segment_id(&self) -> SegmentId {
        match self {
            Event::Start(i) => *i,
            Event::End(i) => *i,
        }
    }
    pub fn shift_segment(&self, segment: &mut Segment, shift: isize) {
        match self {
            Event::Start(_) => segment.shift_start(shift),
            Event::End(_) => segment.shift_end(shift),
        };
    }
    pub fn set_segment(&self, segment: &mut Segment, val: usize) {
        match self {
            Event::Start(_) => segment.set_start(val),
            Event::End(_) => segment.set_end(val),
        };
    }
}
