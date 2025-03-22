use super::{clicqued::ClicquedLinearAxis, event::Event};

pub type CompressedState = Vec<u8>;

pub struct NormalizedState(pub Vec<Event>);

impl ClicquedLinearAxis {
    pub fn strategy_normalize(&self) -> NormalizedState {
        strategy_normalize(&self.inner.events.iter().copied().collect::<Vec<_>>(), self.max_colours)
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

pub fn decompress_to_strategy(max_colours: usize, compressed: &[u8]) -> NormalizedState {
    let uncompressed = decompress(compressed);
    let mut started = vec![None; max_colours];
    fn minimal_started(started: &Vec<Option<usize>>) -> usize {
        started
            .into_iter()
            .enumerate()
            .filter_map(|(c, i)| i.map(|i: usize| (c, i)))
            .min_by(|a, b| a.1.cmp(&b.1))
            .unwrap_or_else(|| panic!("{started:?}"))
            .0
    }
    let mut events = vec![];
    let mut count = 0;
    let mut started_count = 0;
    for e in &uncompressed {
        dbg!(&started, started_count, count);
        if count > 0 && started_count > count {
            break
        }
        if count >= max_colours  {
            break;
        }
        if *e == 0 {
            started[count] = Some(count);
            count += 1;
        } else {
            started_count += 1;
        }
    }
    let len = uncompressed.len();
    println!("{uncompressed:?}");
    for (i, e) in uncompressed.into_iter().enumerate() {
        if e == 0 {
            let c = minimal_started(&started);
            events.push(Event::new_end(c as u8));
            started[c] = None
        } else {
            let c = e - 1;
            events.push(Event::new_start(c));
            started[c as usize] = Some(i + len)
        }
    }
    strategy_normalize(&events, max_colours)
}

pub fn strategy_normalize(events: &[Event], max_colours: usize) -> NormalizedState {
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
