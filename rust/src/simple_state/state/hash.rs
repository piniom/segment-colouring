use super::*;
use std::{
    hash::{Hash, Hasher},
    mem,
};

impl<const MAX_CLIQUE: u32> Hash for State<MAX_CLIQUE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes = unsafe {
            std::slice::from_raw_parts((self as *const Self) as *const u8, mem::size_of::<Self>())
        };
        state.write(bytes);
    }
}
