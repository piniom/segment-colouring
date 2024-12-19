pub use event::Event;
pub use segment::Segment;
use std::{
    collections::{HashMap, HashSet},
    mem::swap,
    ops::{Range, RangeInclusive},
    usize,
};

pub mod event;
pub mod print;
pub mod segment;

pub type SegmentId = u32;

#[derive(Debug, Clone)]
pub struct Axis {
    events: Vec<Event>,
    intersections: Vec<u32>,
    max_clicque: u32,
    pub(crate) segments: HashMap<SegmentId, Segment>,
    counter: SegmentId,
}

impl Default for Axis {
    fn default() -> Self {
        Axis::new(Axis::DEFAULT_MAX_CLICQUE)
    }
}

impl Axis {
    const DEFAULT_MAX_CLICQUE: u32 = 5;
    pub fn new(max_clicque: u32) -> Self {
        Axis {
            events: vec![],
            intersections: vec![],
            max_clicque,
            segments: HashMap::new(),
            counter: 0,
        }
    }
    pub fn insert_segment(&mut self, start_index: usize, end_index: usize) -> Option<SegmentId> {
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

        self.count_intersections();

        Some(id)
    }

    pub fn remove_segment(&mut self, id: u32) -> bool {
        let segment = match self.segments.remove(&id) {
            Some(s) => s,
            None => return false,
        };
        self.remove_maybe_event(segment.end_index());
        self.remove_maybe_event(segment.start_index());

        self.count_intersections();

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

    pub fn confine(&mut self, range: Range<usize>) -> bool {
        let mut old = self.events[range.clone()].to_vec();
        swap(&mut old, &mut self.events);
        for e in &old[0..range.start] {
            self.remove_event_from_segment(e);
        }
        for e in &old[range.end..] {
            self.remove_event_from_segment(e);
        }

        self.count_intersections();
        !self.events.is_empty()
    }

    pub fn confine_left(&mut self) -> bool {
        if self.events().len() == 0 {
            return false;
        }
        self.confine(1..self.events.len())
    }

    pub fn confine_right(&mut self) -> bool {
        if self.events().len() == 0 {
            return false;
        }
        self.confine(0..self.events.len() - 1)
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

    fn count_intersections(&mut self) {
        let mut current = 0;
        let mut result = vec![];
        for e in &self.events {
            if let Event::End(_) = e {
                if self
                    .segments
                    .get(&e.segment_id())
                    .unwrap()
                    .start_index()
                    .is_none()
                {
                    current += 1;
                } else {
                    break;
                }
            }
        }
        for e in &self.events {
            match e {
                Event::Start(_) => {
                    result.push(current);
                    current += 1;
                }
                Event::End(_) => {
                    result.push(current);
                    current -= 1
                }
            }
        }
        self.intersections = result;
    }

    pub fn possible_ends(&self, start_index: usize) -> RangeInclusive<usize> {
        if !self.valid_indexes().any(|i| i == start_index) {
            return 1..=0;
        }
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

        let (a, b) = self
            .valid_indexes()
            .filter(|i| (min_end..=max_end).contains(i))
            .fold((usize::MAX, usize::MIN), |(b, t), i| (b.min(i), t.max(i)));
        a..=b
    }

    pub fn valid_indexes<'a>(&'a self) -> impl Iterator<Item = usize> + use<'a> {
        (0..=self.intersections.len()).filter(|&i| {
            i == self.intersections.len() || self.intersections[i] + 1 <= self.max_clicque
        })
    }

    pub fn segment_collides_with(&self, id: SegmentId) -> Option<impl Iterator<Item = SegmentId>> {
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
