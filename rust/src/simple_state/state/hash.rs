use super::*;
use std::{
    hash::{Hash, Hasher},
};

impl<const MAX_CLIQUE: u32> Hash for State<MAX_CLIQUE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(self.data);
    }
}
