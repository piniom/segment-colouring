use gapbuffer::GapBuffer;
use std::{collections::BTreeMap, f32::consts::PI};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Segment {
    start: f32,
    end: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Event {
    Start(usize),
    End(usize),
}

impl Event {
    pub fn segment_index(&self) -> usize {
        match self {
            Event::Start(i) => *i,
            Event::End(i) => *i,
        }
    }
}

#[derive(Debug, Clone)]
struct Axle {
    events: GapBuffer<Event>,
    counter: usize
}

impl Default for Axle {
    fn default() -> Self {
        Self {
            events: GapBuffer::new(),
            counter: 0
        }
    }
}

impl Axle {
    fn insert_segment(&mut self, start_index: usize, end_index: usize) -> usize {
        let id = self.counter;
        self.counter += 1;
        self.events.insert(start_index, Event::Start(id));
        self.events.insert(end_index + 1, Event::End(id));
        id
    }
}

fn main() {
    println!("Hello, world!");
}
