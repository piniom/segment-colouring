use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct Segment {
    pub start_index: usize,
    pub end_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Start(u32),
    End(u32),
}

impl Event {
    pub fn segment_id(&self) -> u32 {
        match self {
            Event::Start(i) => *i,
            Event::End(i) => *i,
        }
    }
    pub fn ev_type(&self) -> EventType {
        match self {
            Event::Start(_) => EventType::Start,
            Event::End(_) => EventType::End,
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub enum EventType {
    Start,
    End,
}

#[derive(Debug, Clone, Default)]
pub struct Axis {
    events: Vec<Event>,
    segments: HashMap<u32, Segment>,
    counter: u32,
}

impl Axis {
    pub fn insert_segment(&mut self, start_index: usize, end_index: usize) -> Option<u32> {
        if !self.possible_ends(start_index).contains(&end_index) {
            return None;
        }
        let id = self.counter;
        self.counter += 1;
        self.segments.insert(
            id,
            Segment {
                start_index,
                end_index: end_index + 1,
            },
        );
        for e in self.events[start_index..end_index].iter() {
            Self::shift_segment_for_event(&mut self.segments, e, 1);
        }
        for e in self.events[end_index..].iter() {
            Self::shift_segment_for_event(&mut self.segments, e, 2);
        }
        self.events = [
            &self.events[..start_index],
            [Event::Start(id)].as_slice(),
            &self.events[start_index..end_index],
            [Event::End(id)].as_slice(),
            &self.events[end_index..],
        ]
        .join([].as_slice());
        Some(id)
    }

    pub fn remove_segment(&mut self, id: u32) -> bool {
        let segment = match self.segments.remove(&id) {
            Some(s) => s,
            None => return false,
        };
        let start_index = segment.start_index;
        let end_index = segment.end_index;
        for e in self.events[start_index + 1..end_index].iter() {
            Self::shift_segment_for_event(&mut self.segments, e, -1);
        }
        for e in self.events[end_index + 1..].iter() {
            Self::shift_segment_for_event(&mut self.segments, e, -2);
        }
        self.events = [
            &self.events[..start_index],
            &self.events[start_index + 1..end_index],
            &self.events[end_index + 1..],
        ]
        .join([].as_slice());
        true
    }

    pub fn possible_ends(&self, start_index: usize) -> RangeInclusive<usize> {
        let min_end = self
            .segments
            .values()
            .filter(|s| s.start_index < start_index)
            .map(|s| s.end_index + 1)
            .max()
            .unwrap_or(start_index);
        let max_end = self
            .segments
            .values()
            .filter(|s| s.start_index >= start_index)
            .map(|s| s.end_index)
            .min()
            .unwrap_or(start_index.max(min_end));
        min_end..=max_end
    }

    pub fn segment_collides_with(&self, id: u32) -> Option<impl Iterator<Item = u32>> {
        let segment = self.segments.get(&id)?;
        let set: HashSet<_> = self.events[segment.start_index + 1..segment.end_index]
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

    fn shift_segment_for_event(segments: &mut HashMap<u32, Segment>, event: &Event, shift: isize) {
        let segment = segments.get_mut(&event.segment_id()).unwrap();
        match event {
            Event::Start(_) => {
                if shift >= 0 {
                    segment.start_index += shift as usize;
                } else {
                    segment.start_index -= (-shift) as usize;
                }
            }
            Event::End(_) => {
                if shift >= 0 {
                    segment.end_index += shift as usize;
                } else {
                    segment.end_index -= (-shift) as usize;
                }
            }
        };
    }
}
