use crate::simple_state::state::{State};

impl<const MAX_CLIQUE: u32>  State<MAX_CLIQUE> {
    pub fn generate_all(&self, depth: usize) -> Vec<Self> {
        if depth == 0 {
            let mut clone = self.clone();
            clone.normalize();
            return vec![clone];
        }
        let (ends_start, ends_end) = self.valid_segment_ends(0);
        let mut states = vec![];
        for end in ends_start..ends_end {
            let allowed = self.allowed_colours_for_segment(0, end);
            for c in 0..Self::EXPECTED_COLOURS.min(self.colours_used() + 1) {
                if !(allowed >> c) & 0b1 == 1 {
                    continue;
                }
                let mut cloned = self.clone();
                cloned.insert_segment(0, end, c as u8);
                states.extend(cloned.generate_all(depth - 1));
            }
        }
        states
    }
}
