// Each `Event` is 4 bits,
// 0 - 7 for start events (with colours) (first bit is 0 for start events)
// 8 - 15 for end events (with colours) (first bit is 1 for end events)
// We can store 32 events in u128
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub data: u128,
    pub len: usize,
    pub limit_front: usize,
    pub limit_back: usize,
}

const SEGMENT_END: u8 = 0b1111; // 15

impl State {
    pub fn new() -> Self {
        Self {
            data: 0,
            len: 0,
            limit_front: 0,
            limit_back: 0,
        }
    }
    // Assumes that the segment is within the limits
    pub fn insert_segment(&mut self, segment_start: usize, segment_end: usize, color: u8) {
        self.insert_at_indexes(segment_start, color, segment_end, color | 0b1000);
        self.limit_back += 2;
    }
    pub fn remove_segment(&mut self, segment_start: usize, segment_end: usize) {
        self.remove_at_index(segment_end);
        self.remove_at_index(segment_start);
        self.limit_back -= 1;
        if self.limit_back >= segment_end {
            self.limit_back -= 1;
        }
        if self.limit_front >= segment_start {
            self.limit_front -= 1;
        }
    }
    pub fn move_limit_front(&mut self) {
        let first_end = self.find_first_end().unwrap();
        self.remove_at_index(first_end);
        self.remove_at_index(0);
        if self.limit_front > 0 {
            self.limit_front -= 1;
        }
    }
    pub fn move_limit_back(&mut self) {
        let last_start = self.find_last_start().unwrap();
        self.remove_at_index(last_start);
        if self.limit_back == self.len {
            self.limit_back -= 1;
        }
        self.remove_at_index(self.len - 1);
        if self.limit_back > 0 {
            self.limit_back -= 1;
        }
    }
    pub fn normalize(&mut self) {
        let mut color_map = [0u8; 15];
        let mut next_color = 1;
        for i in 0..self.len {
            let value = self.get_at_index(i);
            if value & 0b1000 == 0 {
                if color_map[value as usize] == 0 {
                    color_map[value as usize] = next_color;
                    next_color += 1;
                }
                self.replace_at_index(i, color_map[value as usize] - 1);
            } else {
                self.replace_at_index(i, color_map[(value & 0b111) as usize] - 1 | 0b1000);
            }
        }
    }
    pub fn flip(&mut self) {
        for i in 0..(self.len + 1 / 2) {
            let j = self.len - 1 - i;
            let left = self.get_at_index(i);
            let right = self.get_at_index(j);
            self.replace_at_index(i, right ^ 0b1000);
            self.replace_at_index(j, left ^ 0b1000);
        }
        self.limit_front = self.len - self.limit_back;
        self.limit_back = self.len - self.limit_front - 1;
    }
    fn find_first_end(&self) -> Option<usize> {
        for i in self.limit_front..self.len {
            if self.get_at_index(i) & 0b1000 != 0 {
                return Some(i);
            }
        }
        None
    }
    fn find_last_start(&self) -> Option<usize> {
        for i in (0..=self.limit_back).rev() {
            if self.get_at_index(i) & 0b1000 == 0 {
                return Some(i);
            }
        }
        None
    }
    #[inline(always)]
    fn replace_at_index(&mut self, index: usize, value: u8) {
        let shift = index * 4;
        self.data &= !(0b1111 << shift);
        self.data |= (value as u128) << shift;
    }
    #[inline(always)]
    fn remove_at_index(&mut self, index: usize) {
        let shift = index * 4;
        self.data &= !(0b1111 << shift);
        self.len -= 1;
    }
    #[inline(always)]
    fn get_at_index(&self, index: usize) -> u8 {
        let shift = index * 4;
        ((self.data >> shift) & 0b1111) as u8
    }
    #[inline(always)]
    // Shifts all events starting from index to the right and inserts the new value at index
    fn insert_at_index(&mut self, index: usize, value: u8) {
        let shift = index * 4;
        let mask = (1u128 << shift) - 1;
        let upper = self.data & !mask;
        let lower = self.data & mask;
        self.data = upper << 4 | (value as u128) << shift | lower;
        self.len += 1;
    }
    #[inline(always)]
    fn insert_at_indexes(&mut self, index_a: usize, value_a: u8, index_b: usize, value_b: u8) {
        self.insert_at_index(index_b, value_b);
        self.insert_at_index(index_a, value_a);
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_state() {
        let mut state = State::new();
        state.insert_at_index(0, 1);
        state.insert_at_index(1, 2);
        state.insert_at_index(2, 3);
        state.insert_at_index(3, 4);
        assert_eq!(state.get_at_index(0), 1);
        assert_eq!(state.get_at_index(1), 2);
        assert_eq!(state.get_at_index(2), 3);
        assert_eq!(state.get_at_index(3), 4);
    }
    #[test]
    fn test_state_insert_at_indexes() {
        let mut state = State::new();
        state.insert_at_indexes(0, 1, 0, 2);
        state.insert_at_indexes(0, 3, 1, 4);
        assert_eq!(state.get_at_index(0), 3);
        assert_eq!(state.get_at_index(1), 1);
        assert_eq!(state.get_at_index(2), 4);
        assert_eq!(state.get_at_index(3), 2);
    }
    #[test]
    fn test_state_flip() {
        let mut state = State::new();
        state.insert_at_indexes(0, 1, 0, 2);
        state.insert_at_indexes(0, 3, 1, 4);
        let clone = state.clone();
        state.flip();
        state.flip();
        assert_eq!(state, clone);
    }
}
