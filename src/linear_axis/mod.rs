use std::{collections::VecDeque, fmt::Debug};

use serde::{Deserialize, Serialize};

pub mod clicqued;
pub mod print;
pub mod normalization;
pub mod game;

#[derive(Debug, Clone)]
pub struct LinearAxis {
    pub events: VecDeque<Event>,
}

impl LinearAxis {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }
    fn apply_history(&mut self, history: History) -> Option<History> {
        match history {
            History::SegmentInsert {
                start_index,
                end_index,
                color,
            } => {
                self.insert_segment(start_index, end_index, color);
                Some(History::SegmentRemove {
                    start_index: start_index,
                    end_index: end_index + 1,
                })
            }
            History::SegmentRemove {
                start_index,
                end_index,
            } => self
                .remove_segment(start_index, end_index)
                .map(|c| History::SegmentInsert {
                    start_index: start_index,
                    end_index: end_index - 1,
                    color: c,
                }),
            History::LimitFront => self.limit_front().map(|e| History::EventInsertFront(e)),
            History::LimitBack => self.limit_back().map(|e| History::EventInsertBack(e)),
            History::EventInsertFront(event) => {
                self.insert_event_front(event);
                Some(History::LimitFront)
            }
            History::EventInsertBack(event) => {
                self.insert_event_back(event);
                Some(History::LimitBack)
            }
        }
    }
    fn insert_segment(&mut self, start_index: usize, end_index: usize, color: u8) {
        let mut new_events = VecDeque::with_capacity(self.events.len() + 2);
        new_events.extend(self.events.iter().take(start_index).cloned());
        new_events.push_back(Event::new(true, color));
        new_events.extend(
            self.events
                .iter()
                .skip(start_index)
                .take(end_index - start_index)
                .cloned(),
        );
        new_events.push_back(Event::new(false, color));
        new_events.extend(self.events.iter().skip(end_index).cloned());
        self.events = new_events;
    }
    fn remove_segment(&mut self, start_index: usize, end_index: usize) -> Option<u8> {
        let mut new_events = VecDeque::with_capacity(self.events.len() - 2);
        new_events.extend(self.events.iter().take(start_index).cloned());

        let start_color = self.events.get(start_index)?.colour();
        let end_color = self.events.get(end_index)?.colour();
        if start_color != end_color {
            return None;
        }

        new_events.extend(
            self.events
                .iter()
                .skip(start_index + 1)
                .take(end_index - start_index - 1)
                .cloned(),
        );

        new_events.extend(self.events.iter().skip(end_index + 1).cloned());
        self.events = new_events;

        Some(start_color)
    }
    fn limit_front(&mut self) -> Option<Event> {
        self.events.pop_front()
    }
    fn limit_back(&mut self) -> Option<Event> {
        self.events.pop_back()
    }
    fn insert_event_front(&mut self, event: Event) {
        self.events.push_front(event);
    }
    fn insert_event_back(&mut self, event: Event) {
        self.events.push_back(event);
    }
}

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

#[test]
fn test_linear_axis_history() {
    let mut axis = LinearAxis::new();
    let moves = vec![
        History::SegmentInsert {
            start_index: 0,
            end_index: 0,
            color: 1,
        },
        History::SegmentInsert {
            start_index: 0,
            end_index: 1,
            color: 2,
        },
        History::LimitFront,
        History::SegmentInsert { start_index: 2, end_index: 3, color: 3 }
    ];
    let mut history = vec![];
    let mut reconstruct = vec![];
    for m in &moves {
        let r = axis.apply_history(*m).unwrap();
        history.push(r);
    }
    for h in history {
        let m = axis.apply_history(h).unwrap();
        reconstruct.push(m);
    }
    assert_eq!(moves, reconstruct);
    assert_eq!(axis.events, LinearAxis::new().events);
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event(u8);

impl Event {
    pub fn new_start(color: u8) -> Self {
        Self::new(true, color)
    }
    pub fn new_end(color: u8) -> Self {
        Self::new(false, color)
    }
    fn new(is_start: bool, color: u8) -> Self {
        Event((is_start as u8) | (color << 1))
    }
    pub fn is_start(&self) -> bool {
        (self.0 & 1) != 0
    }
    pub fn colour(&self) -> u8 {
        self.0 >> 1
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("is_start", &self.is_start())
            .field("color", &self.colour())
            .finish()
    }
}
