use std::collections::VecDeque;

use super::{clicqued::ClicquedLinearAxis, event::Event, strategy::StrategyMove};

pub type CompressedState = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedState(pub Vec<Event>);

impl ClicquedLinearAxis {
    pub fn strategy_normalize(&mut self) -> (NormalizedState, bool) {
        self.normalizer.strategy_normalize(
            &self.inner.events.iter().copied().collect::<Vec<_>>(),
            self.max_colors(),
        )
    }

    pub fn strategy_normalize_without_symmetry(&mut self) -> NormalizedState {
        self.normalizer.strategy_normalize_without_symmetry(
            &self.inner.events.iter().copied().collect::<Vec<_>>(),
            self.max_colors(),
        )
    }

    pub fn strategy_normalize_with_move(
        &mut self,
        mov: StrategyMove,
    ) -> (NormalizedState, StrategyMove) {
        let (norm, flip) = self.strategy_normalize();
        if flip {
            (norm, self.flip_move(mov))
        } else {
            (norm, mov)
        }
    }

    fn flip_move(&self, mov: StrategyMove) -> StrategyMove {
        match mov {
            StrategyMove::LimitBack => StrategyMove::LimitFront,
            StrategyMove::LimitFront => StrategyMove::LimitBack,
            StrategyMove::Insert { start, end } => {
                let len = self.inner.events.len();
                StrategyMove::Insert {
                    start: len - end,
                    end: len - start,
                }
            }
        }
    }
     pub fn normalize_compress(&self) -> Vec<u8> {
        let mut colours = vec![u8::MAX; self.max_colors()];
        let mut normalized = vec![];
        let mut i = 1;
        let mut max_dis: u8 = 0;
        for e in &self.inner.events {
            if colours[e.colour() as usize] == u8::MAX {
                colours[e.colour() as usize] = i;
                i += 1;
                if !e.is_start() {
                    max_dis += 1;
                }
            }
            if e.is_start() {
                normalized.push(colours[e.colour() as usize]);
            } else {
                normalized.push(0);
            }
        }
        let mut compressed = compress(&normalized);
        compressed.push(max_dis);
        compressed
    }
}

fn compress(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::with_capacity((data.len() + 1) / 2);

    for chunk in data.chunks(2) {
        let first = chunk[0] & 0x0F;
        let second = if chunk.len() > 1 {
            (chunk[1] + 1) & 0x0F
        } else {
            0
        };

        compressed.push((first << 4) | second);
    }

    compressed
}

#[derive(Debug, Clone)]
pub struct StrategyNormalizer {
    colours: [u8; 20],
}
impl StrategyNormalizer {
    pub fn new() -> Self {
        Self {
            colours: [u8::MAX; 20],
        }
    }
    pub fn decompress_to_strategy(
        &mut self,
        max_colours: usize,
        compressed: &[u8],
        mov: StrategyMove,
    ) -> (NormalizedState, StrategyMove) {
        let mut max_dis = *compressed.last().unwrap();
        let uncompressed = decompress(&compressed[0..compressed.len() - 1]);
        let mut queue = VecDeque::new();
        let mut events = vec![];

        // println!("{uncompressed:?}");

        for (i, e) in uncompressed.iter().enumerate() {
            if max_dis == 0 {
                break;
            }
            if *e == 0 {
                queue.push_back((i) as u8);
                max_dis -= 1;
            }
        }

        for e in uncompressed {
            if e == 0 {
                events.push(Event::new_end(queue.pop_front().unwrap()));
            } else {
                let c = e - 1;
                queue.push_back(c);
                events.push(Event::new_start(c));
            }
        }

        // println!("{:?}", events.iter().map(Event::to_char).collect::<Vec<_>>());

        let (state, flip) = self.strategy_normalize(&events, max_colours);

        // println!("{:?}\n", strategy.0.iter().map(Event::to_char).collect::<Vec<_>>());
        if flip {
            (
                state,
                match mov {
                    StrategyMove::LimitBack => StrategyMove::LimitFront,
                    StrategyMove::LimitFront => StrategyMove::LimitBack,
                    _ => panic!(),
                },
            )
        } else {
            (state, mov)
        }
    }
    pub fn strategy_normalize_without_symmetry(
        &mut self,
        events: &[Event],
        max_colours: usize,
    ) -> NormalizedState {
        for i in 0..max_colours {
            self.colours[i] = u8::MAX
        }

        let mut normalized = Vec::with_capacity(events.len());
        let mut i = 0;
        for e in events.iter().filter(|e| !e.is_start()) {
            if self.colours[e.colour() as usize] == u8::MAX {
                self.colours[e.colour() as usize] = i;
                i += 1;
            }
        }
        for e in events {
            if self.colours[e.colour() as usize] == u8::MAX {
                self.colours[e.colour() as usize] = i;
                i += 1;
            }
            normalized.push(e.with_color(self.colours[e.colour() as usize]));
        }
        NormalizedState(normalized)
    }
    pub fn strategy_normalize(
        &mut self,
        events: &[Event],
        max_colours: usize,
    ) -> (NormalizedState, bool) {
        let base = self.strategy_normalize_without_symmetry(events, max_colours);
        let flipped = base.flipped(self, max_colours);
        match base.cmp(&flipped) {
            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => (base, false),
            std::cmp::Ordering::Less => (flipped, true),
        }
    }
}

fn decompress(data: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::with_capacity(data.len() * 2);

    for &byte in data {
        let first = byte >> 4;
        let second = byte & 0x0F;

        decompressed.push(first);
        if second != 0 {
            decompressed.push(second - 1);
        }
    }

    decompressed
}

// #[test]
// fn test_hybydy() {
//     let compressed = compress(&[1, 2, 0, 0]);
//     assert_eq!(
//         "ABab",
//         decompress_to_strategy(5, &compressed, StrategyMove::LimitBack)
//             .0
//              .0
//             .iter()
//             .map(Event::to_char)
//             .collect::<String>()
//     )
// }

impl NormalizedState {
    pub fn flipped(&self, normalizer: &mut StrategyNormalizer, max_colours: usize) -> Self {
        normalizer.strategy_normalize_without_symmetry(
            &self.0.iter().map(Event::sibling).rev().collect::<Vec<_>>(),
            max_colours,
        )
    }
    pub fn colors_used(&self) -> usize {
        let mut used = vec![false; self.0.len()];
        for e in &self.0 {
            used[e.colour() as usize] = true;
        }
        used.into_iter().filter(|v| *v).count()
    }
}
