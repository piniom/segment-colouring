use super::*;
use std::{
    hash::{Hash, Hasher},
    mem,
};

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes = unsafe {
            std::slice::from_raw_parts((self as *const Self) as *const u8, mem::size_of::<Self>())
        };
        state.write(bytes);
    }
}
