use std::{collections::VecDeque, fmt::Debug};

use event::Event;
use history::History;

pub mod clicqued;
pub mod event;
pub mod game;
pub mod history;
pub mod normalization;
pub mod print;
pub mod strategy;

#[derive(Debug, Clone)]
pub struct LinearAxis {
    pub events: VecDeque<Event>,
    pub max_colors: usize,
    pub front: VecDeque<Event>,
    pub back: VecDeque<Event>
}

impl LinearAxis {
    fn new(max_colors: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_colors,
            front: VecDeque::new(),
            back: VecDeque::new(),
        }
    }
    fn apply_history(&mut self, history: History) -> Option<History> {
        match history {
            History::SegmentInsert {
                start_index,
                end_index,
                color,
            } => {
                self.insert_segment(start_index, end_index, color)?;
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
            History::LimitFront => self
                .limit_front()
                .map(|(end, lost)| History::EventInsertFront { end, lost}),
            History::LimitBack => self
                .limit_back()
                .map(|(start, lost)| History::EventInsertBack { start,lost }),
            History::EventInsertFront {  end, lost } => {
                self.insert_event_front(end, lost);
                Some(History::LimitFront)
            }
            History::EventInsertBack{ start, lost } => {
                self.insert_event_back(start, lost);
                Some(History::LimitBack)
            }
        }
    }
    fn insert_segment(&mut self, start_index: usize, end_index: usize, color: u8) -> Option<()> {
        if start_index > self.events.len() || end_index > self.events.len() {
            return None;
        }
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
        Some(())
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
    fn limit_front(&mut self) -> Option<(Event, usize)> {
        let mut started = vec![false; self.max_colors];
        let mut found = None;
        for (i, e) in self.events.iter().enumerate() {
            if e.is_start() {
                started[e.colour() as usize] = true
            } else {
                found = Some(i);
                break;
            }
        }
        let found = found?;
        for _ in 0..found {
            self.front.push_back(self.events.pop_front().unwrap());
        }
        let end = self.events.pop_front()?;
        Some((end, found))
    }
    fn limit_back(&mut self) -> Option<(Event, usize)> {
        let mut ended = vec![false; self.max_colors];
        let mut found = None;
        for (i, e) in self.events.iter().rev().enumerate() {
            if e.is_start() {
                found = Some(i);
                break;
            } else {
                ended[e.colour() as usize] = true
            }
        }
        let found = found?;
        for _ in 0..found {
            self.back.push_front(self.events.pop_back().unwrap());
        }
        let start = self.events.pop_back()?;
        Some((start, found))
    }
    fn insert_event_front(&mut self, end: Event, lost: usize) {
        self.events.push_front(end);
        for _ in 0..lost {
            self.events.push_front(self.front.pop_back().unwrap());
        }
    }
    fn insert_event_back(&mut self, start: Event, lost: usize) {
        self.events.push_back(start);
        for _ in 0..lost {
            self.events.push_back(self.back.pop_front().unwrap());
        }
    }
}

#[test]
fn test_linear_axis_history() {
    let mut axis = LinearAxis::new(4);
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
        History::LimitBack,
        History::SegmentInsert {
            start_index: 0,
            end_index: 1,
            color: 3,
        },
        History::LimitFront
    ];
    let mut history = vec![];
    let mut reconstruct = vec![];
    for m in &moves {
        println!("f: {:?}", m);
        let r = axis.apply_history(*m).unwrap();
        println!("{}", axis.to_string());
        println!("r: {:?}\n", r);
        history.push(r);
    }
    println!("{}", axis.to_string());
    for h in history.iter().rev() {
        println!("r: {:?}", h);
        let m = axis.apply_history(*h).unwrap();
        println!("{}\n", axis.to_string());
        reconstruct.push(m);
    }
    assert_eq!(moves, reconstruct.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(axis.events, LinearAxis::new(4).events);
}

#[test]
fn test_linear_axis_history_reduction() {
    use normalization::strategy_normalize;
    let mut axis = LinearAxis {
        events: vec![Event::new_start(1), Event::new_end(0), Event::new_end(1), Event::new_start(1)].into(),
        max_colors: 3,
        front: vec![Event::new_start(0)].into(),
        back: vec![Event::new_end(1)].into(),
    };
    axis.apply_history(History::LimitFront);
    assert_eq!(strategy_normalize(&axis.events.into_iter().collect::<Vec<_>>(), 3).0, vec![Event::new_end(0), Event::new_start(0)])
}

#[test]
fn test_linear_axis_history_reduction_2() {
    use normalization::strategy_normalize;
    let mut axis = LinearAxis {
        events: vec![Event::new_start(1), Event::new_end(0)].into(),
        max_colors: 3,
        front: vec![Event::new_start(0)].into(),
        back: vec![Event::new_end(1)].into(),
    };
    axis.apply_history(History::LimitBack);
    assert_eq!(strategy_normalize(&axis.events.into_iter().collect::<Vec<_>>(), 3).0, vec![])
}
