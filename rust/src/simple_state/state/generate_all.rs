use crate::simple_state::state::{State};

impl<const MAX_CLIQUE: u32>  State<MAX_CLIQUE> {
    pub fn generate_all(&self, depth: usize) -> Vec<Self> {
        if depth == 0 {
            let mut clone = self.clone();
            clone.normalize();
            return vec![clone];
        }
        let mut states = vec![];
        for move_ in self.front_moves() {
            for new_state in move_.outcomes() {
                states.extend(new_state.generate_all(depth - 1));
            }
        }
        states
    }
}
