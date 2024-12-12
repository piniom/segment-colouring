use std::{
    collections::{HashMap, HashSet},
    mem::swap,
    ops::RangeInclusive,
    usize,
};

pub type SegmentId = u32;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct Ssegment {
    pub start_index: usize,
    pub end_index: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct Segment {
    start_index: Option<usize>,
    end_index: Option<usize>,
}

impl Segment {
    pub fn new(start_index: usize, end_index: usize) -> Self {
        Segment {
            start_index: Some(start_index),
            end_index: Some(end_index),
        }
    }
    pub fn left(end_index: usize) -> Self {
        Segment {
            start_index: None,
            end_index: Some(end_index),
        }
    }
    pub fn right(start_index: usize) -> Self {
        Segment {
            start_index: Some(start_index),
            end_index: None,
        }
    }
    pub fn shift_start(&mut self, shift: isize) {
        let mut start = match self.start_index() {
            Some(s) => s,
            None => return,
        };
        if shift >= 0 {
            start += shift as usize;
        } else {
            start -= (-shift) as usize;
        }
        self.set_start(start)
    }
    pub fn remove_start(&mut self) {
        self.start_index = None;
    }
    pub fn remove_end(&mut self) {
        self.end_index = None;
    }
    pub fn shift_end(&mut self, shift: isize) {
        let mut end = match self.end_index() {
            Some(s) => s,
            None => return,
        };
        if shift >= 0 {
            end += shift as usize;
        } else {
            end -= (-shift) as usize;
        }
        self.set_end(end)
    }
    pub fn set_start(&mut self, start_index: usize) {
        self.start_index = Some(start_index)
    }
    pub fn set_end(&mut self, end_index: usize) {
        self.end_index = Some(end_index)
    }
    pub fn starts_before(&self, other: usize) -> bool {
        match self.start_index {
            Some(s) => s < other,
            None => true,
        }
    }
    pub fn start_index(&self) -> Option<usize> {
        self.start_index
    }
    pub fn end_index(&self) -> Option<usize> {
        self.end_index
    }
    pub fn has_both_events(&self) -> bool {
        self.start_index.is_some() && self.end_index.is_some()
    }
}

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
}

#[derive(Debug, Clone, Default)]
pub struct Axis {
    events: Vec<Event>,
    segments: HashMap<SegmentId, Segment>,
    counter: SegmentId,
}

impl Axis {
    pub fn insert_segment(&mut self, start_index: usize, end_index: usize) -> Option<u32> {
        if !self.possible_ends(start_index).contains(&end_index) {
            return None;
        }
        let id = self.counter;

        self.counter += 1;
        self.segments
            .insert(id, Segment::new(start_index, end_index + 1));

        for e in self.events[start_index..end_index].iter() {
            e.shift_segment(self.segments.get_mut(&e.segment_id()).unwrap(), 1);
        }
        for e in self.events[end_index..].iter() {
            e.shift_segment(self.segments.get_mut(&e.segment_id()).unwrap(), 2);
        }

        let mut new_events = Vec::with_capacity(self.events.len() + 2);
        new_events.extend_from_slice(&self.events[..start_index]);
        new_events.push(Event::Start(id));
        new_events.extend_from_slice(&self.events[start_index..end_index]);
        new_events.push(Event::End(id));
        new_events.extend_from_slice(&self.events[end_index..]);
        self.events = new_events;

        Some(id)
    }

    pub fn remove_segment(&mut self, id: u32) -> bool {
        let segment = match self.segments.remove(&id) {
            Some(s) => s,
            None => return false,
        };
        self.remove_maybe_event(segment.end_index());
        self.remove_maybe_event(segment.start_index());

        true
    }

    fn remove_maybe_event(&mut self, index: Option<usize>) -> bool {
        match index {
            Some(i) => {
                self.remove_event(i);
                true
            }
            None => false,
        }
    }

    fn remove_event(&mut self, index: usize) {
        for e in self.events[index + 1..].iter() {
            e.shift_segment(self.segments.get_mut(&e.segment_id()).unwrap(), -1);
        }
        let mut new_events = Vec::with_capacity(self.events.len() - 1);
        new_events.extend_from_slice(&self.events[..index]);
        new_events.extend_from_slice(&self.events[index + 1..]);
        self.events = new_events;
    }

    pub fn confine(&mut self, range: RangeInclusive<usize>) {
        let mut old = self.events[range.clone()].to_vec();
        swap(&mut old, &mut self.events);
        for e in &old[0..*range.start()] {
            self.remove_event_from_segment(e);
        }
        for e in &old[*range.end() + 1..] {
            self.remove_event_from_segment(e);
        }
    }

    fn remove_event_from_segment(&mut self, event: &Event) -> Option<()> {
        let id = event.segment_id();
        let s = self.segments.get_mut(&id)?;
        if s.has_both_events() {
            match event {
                Event::Start(_) => s.remove_start(),
                Event::End(_) => s.remove_end(),
            }
        } else {
            self.segments.remove(&id);
        }
        Some(())
    }

    pub fn possible_ends(&self, start_index: usize) -> RangeInclusive<usize> {
        let min_end = self
            .segments
            .values()
            .filter(|s| s.starts_before(start_index))
            .map(|s| s.end_index().unwrap_or(usize::MAX - 1) + 1)
            .max()
            .unwrap_or(start_index);

        let max_end = self
            .segments
            .values()
            .filter(|s| !s.starts_before(start_index))
            .filter_map(|s| s.end_index())
            .min()
            .unwrap_or(min_end);

        min_end..=max_end
    }

    pub fn segment_collides_with(&self, id: u32) -> Option<impl Iterator<Item = u32>> {
        let segment = self.segments.get(&id)?;
        let set: HashSet<_> = self.events[segment.start_index().map(|i| i + 1).unwrap_or_default()
            ..segment.end_index().unwrap_or(usize::MAX)]
            .iter()
            .map(|e| e.segment_id())
            .collect();
        Some(set.into_iter())
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn segments(&self) -> Vec<Segment> {
        let mut arr = self.segments.clone().into_iter().collect::<Vec<_>>();
        arr.sort_by(|(a, _), (b, _)| a.cmp(b));
        arr.into_iter().map(|(_, s)| s).collect()
    }
}
