use super::{clicqued::ClicquedLinearAxis, event::Event};

pub type CompressedState = Vec<u8>;

pub struct NormalizedState(pub Vec<Event>);

impl ClicquedLinearAxis {
    pub fn strategy_normalize(&self) -> NormalizedState {
        let mut colours = vec![u8::MAX; self.max_colours];
        let mut normalized = vec![];
        let mut i = 0;
        for e in self.inner.events.iter().filter(|e| !e.is_start()) { 
            if colours[e.colour() as usize] == u8::MAX {
                colours[e.colour() as usize] = i;
                i += 1;
            }
        }
        for e in &self.inner.events {
            if colours[e.colour() as usize] == u8::MAX {
                colours[e.colour() as usize] = i;
                i += 1;
            }
            normalized.push(e.with_color(colours[e.colour() as usize]));
        }
        NormalizedState(normalized)
    }
    pub fn normalize_compress(&self) -> Vec<u8> {
        let mut colours = vec![u8::MAX; self.max_colours];
        let mut normalized = vec![];
        let mut i = 1;
        for e in &self.inner.events {
            if colours[e.colour() as usize] == u8::MAX {
                colours[e.colour() as usize] = i;
                i += 1;
            }
            if e.is_start() {
                normalized.push(colours[e.colour() as usize]);
            } else {
                normalized.push(0);
            }
        }
        compress(&normalized)
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
