use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct Event(u8);

impl Event {
    pub fn new_start(color: u8) -> Self {
        Self::new(true, color)
    }
    pub fn new_end(color: u8) -> Self {
        Self::new(false, color)
    }
    pub fn with_color(&self, color: u8) -> Self {
        Self::new(self.is_start(), color)
    }
    fn new(is_start: bool, color: u8) -> Self {
        Event((is_start as u8) | (color << 1))
    }
    pub fn is_start(&self) -> bool {
        (self.0 & 1) != 0
    }
    pub fn colour(&self) -> u8 {
        self.0 >> 1
    }
    pub fn to_char(&self) -> char {
        let case = if self.is_start() { 'A' } else { 'a' };
        (case as u8 + self.colour()) as char
    }
    pub fn from_char(c: char) -> Self {
        if c >= 'a' {
            Self::new_end(c as u8 - 'a' as u8)
        } else {
            Self::new_start(c as u8 - 'A' as u8)
        }
    }
    pub fn sibling(&self) -> Self {
        Event(self.0 ^ 1)
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("is_start", &self.is_start())
            .field("color", &self.colour())
            .finish()
    }
}
