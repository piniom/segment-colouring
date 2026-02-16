use event::Event;
use history::History;
use queue::Queue;

pub mod clicqued;
pub mod event;
pub mod game;
pub mod history;
pub mod normalization;
pub mod print;
pub mod queue;
pub mod strategy;

#[derive(Debug, Clone)]
pub struct LinearAxis {
    pub events: Queue<Event>,
    pub front: Queue<Event>,
    pub back: Queue<Event>,
}

impl LinearAxis {
    fn new() -> Self {
        Self {
            events: Queue::new(),
            front: Queue::new(),
            back: Queue::new(),
        }
    }
    fn apply_history(&mut self, history: History, max_colors: usize) -> Option<History> {
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
                .limit_front(max_colors)
                .map(|(end, lost)| History::EventInsertFront { end, lost }),
            History::LimitBack => self
                .limit_back(max_colors)
                .map(|(start, lost)| History::EventInsertBack { start, lost }),
            History::EventInsertFront { end, lost } => {
                self.insert_event_front(end, lost);
                Some(History::LimitFront)
            }
            History::EventInsertBack { start, lost } => {
                self.insert_event_back(start, lost);
                Some(History::LimitBack)
            }
        }
    }
    fn insert_segment(&mut self, start_index: usize, end_index: usize, color: u8) -> Option<()> {
        if start_index > self.events.len() || end_index > self.events.len() {
            return None;
        }
        self.events
            .insert_at_index(end_index, Event::new_end(color));
        self.events
            .insert_at_index(start_index, Event::new_start(color))
    }
    fn remove_segment(&mut self, start_index: usize, end_index: usize) -> Option<u8> {
        let end_color = self.events.remove_at_index(end_index)?.colour();
        let start_color = self.events.remove_at_index(start_index)?.colour();
        if start_color == end_color {
            Some(start_color)
        } else {
            None
        }
    }
    fn limit_front(&mut self, max_colors: usize) -> Option<(Event, usize)> {
        let mut started = vec![false; max_colors];
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
    fn limit_back(&mut self, max_colors: usize) -> Option<(Event, usize)> {
        let mut ended = vec![false; max_colors];
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
        History::LimitBack,
        History::SegmentInsert {
            start_index: 0,
            end_index: 1,
            color: 3,
        },
        History::LimitFront,
    ];
    let mut history = vec![];
    let mut reconstruct = vec![];
    for m in &moves {
        println!("f: {:?}", m);
        let r = axis.apply_history(*m, 4).unwrap();
        println!("{}", axis.to_string());
        println!("r: {:?}\n", r);
        history.push(r);
    }
    println!("{}", axis.to_string());
    for h in history.iter().rev() {
        println!("r: {:?}", h);
        let m = axis.apply_history(*h, 4).unwrap();
        println!("{}\n", axis.to_string());
        reconstruct.push(m);
    }
    assert_eq!(moves, reconstruct.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(axis.events, LinearAxis::new().events);
}

// #[test]
// fn test_linear_axis_history_reduction() {
//     use normalization::strategy_normalize;
//     let mut axis = LinearAxis {
//         events: vec![
//             Event::new_start(1),
//             Event::new_end(0),
//             Event::new_end(1),
//             Event::new_start(1),
//         ]
//         .into(),
//         front: vec![Event::new_start(0)].into(),
//         back: vec![Event::new_end(1)].into(),
//     };
//     axis.apply_history(History::LimitFront, 4);
//     assert_eq!(
//         strategy_normalize(&axis.events.into_iter().collect::<Vec<_>>(), 3)
//             .0
//              .0,
//         vec![Event::new_end(0), Event::new_start(0)]
//     )
// }

// #[test]
// fn test_linear_axis_history_reduction_2() {
//     use normalization::strategy_normalize;
//     let mut axis = LinearAxis {
//         events: vec![Event::new_start(1), Event::new_end(0)].into(),
//         front: vec![Event::new_start(0)].into(),
//         back: vec![Event::new_end(1)].into(),
//     };
//     axis.apply_history(History::LimitBack, 4);
//     assert_eq!(
//         strategy_normalize(&axis.events.into_iter().collect::<Vec<_>>(), 3)
//             .0
//              .0,
//         vec![]
//     )
// }
