use std::{collections::VecDeque, fmt::Debug};

use event::Event;
use history::History;

pub mod clicqued;
pub mod game;
pub mod normalization;
pub mod print;
pub mod history;
pub mod event;

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
        new_events.push_back(Event::new_start(color));
        new_events.extend(
            self.events
                .iter()
                .skip(start_index)
                .take(end_index - start_index)
                .cloned(),
        );
        new_events.push_back(Event::new_end(color));
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
        History::SegmentInsert {
            start_index: 2,
            end_index: 3,
            color: 3,
        },
    ];
    let mut history = vec![];
    let mut reconstruct = vec![];
    for m in &moves {
        let r = axis.apply_history(*m).unwrap();
        history.push(r);
    }
    for h in history.iter().rev() {
        dbg!(h);
        let m = axis.apply_history(*h).unwrap();
        reconstruct.push(m);
    }
    assert_eq!(moves, reconstruct.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(axis.events, LinearAxis::new().events);
}
