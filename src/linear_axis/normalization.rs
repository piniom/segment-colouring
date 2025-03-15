use super::clicqued::ClicquedLinearAxis;

pub type NormalizedState = Vec<u8>;

impl ClicquedLinearAxis {
    pub fn normalize(&self) -> Vec<u8> {
        let mut colours = vec![u8::MAX; self.max_colours];
        let mut normalized = vec![];
        let mut i = 1;
        for e in &self.inner.events {
            if e.is_start() {
                if colours[e.colour() as usize] == u8::MAX {
                    colours[e.colour() as usize] = i;
                    i += 1;
                }
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
