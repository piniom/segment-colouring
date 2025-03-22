use std::collections::VecDeque;

use super::{clicqued::ClicquedLinearAxis, event::Event};

pub type CompressedState = Vec<u8>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalizedState(pub Vec<Event>);

impl ClicquedLinearAxis {
    pub fn strategy_normalize(&self) -> NormalizedState {
        strategy_normalize(
            &self.inner.events.iter().copied().collect::<Vec<_>>(),
            self.max_colours,
        )
    }

    pub fn normalize_compress(&self) -> Vec<u8> {
        let mut colours = vec![u8::MAX; self.max_colours];
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

pub fn decompress_to_strategy(max_colours: usize, compressed: &[u8]) -> NormalizedState {
    let mut max_dis = *compressed.last().unwrap();
    let uncompressed = decompress(&compressed[0..compressed.len()-1]);
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

    let strategy = strategy_normalize(&events, max_colours);

    // println!("{:?}\n", strategy.0.iter().map(Event::to_char).collect::<Vec<_>>());
    strategy
}

pub fn strategy_normalize(events: &[Event], max_colours: usize) -> NormalizedState {
    let base = strategy_normalize_without_symmetry(events, max_colours);
    let flipped = base.flipped(max_colours);
    match base.cmp(&flipped) {
        std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => base,
        std::cmp::Ordering::Less => flipped,
    }
}

pub fn strategy_normalize_without_symmetry(events: &[Event], max_colours: usize) -> NormalizedState {
    let mut colours = vec![u8::MAX; max_colours];
    let mut normalized = vec![];
    let mut i = 0;
    for e in events.iter().filter(|e| !e.is_start()) {
        if colours[e.colour() as usize] == u8::MAX {
            colours[e.colour() as usize] = i;
            i += 1;
        }
    }
    for e in events {
        if colours[e.colour() as usize] == u8::MAX {
            colours[e.colour() as usize] = i;
            i += 1;
        }
        normalized.push(e.with_color(colours[e.colour() as usize]));
    }
    NormalizedState(normalized)
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

#[test]
fn test_hybydy() {
    let compressed = compress(&[1, 2, 0, 0]);
    assert_eq!(
        "ABab",
        decompress_to_strategy(5, &compressed)
            .0
            .iter()
            .map(Event::to_char)
            .collect::<String>()
    )
}

impl NormalizedState {
    fn flipped(&self, max_colours: usize) -> Self {
        strategy_normalize_without_symmetry(
            &self.0.iter().map(Event::sibling).rev().collect::<Vec<_>>(),
            max_colours,
        )
    }
}
