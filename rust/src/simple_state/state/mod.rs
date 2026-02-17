#[cfg(test)]
mod test;

pub mod string;
// Each `Event` is 4 bits,
// 0 - 7 for start events (with colours) (first bit is 0 for start events)
// 8 - 15 for end events (with colours) (first bit is 1 for end events)
// We can store 32 events in u128
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    pub data: u128,
    pub len: usize,
    pub limit_front: usize,
    pub limit_back: usize,
}

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
    }
    pub fn remove_segment(&mut self, segment_start: usize, segment_end: usize) {
        self.remove_at_index(segment_end);
        self.remove_at_index(segment_start);
    }
    pub fn move_limit_front(&mut self) {
        let first_end = self.find_first_end().unwrap();
        dbg!(first_end, self.len);
        self.limit_front = first_end;
        dbg!(&self);
        self.remove_at_index(first_end);
        dbg!(&self);
        self.remove_at_index(0);
    }
    pub fn move_limit_back(&mut self) {
        let last_start = self.find_last_start().unwrap();
        dbg!(last_start, self.len);
        self.limit_back = last_start;
        self.remove_at_index(last_start);
        self.remove_at_index(self.len - 1);
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
    pub fn intersections(&self) -> [usize; 32] {
        let mut cur = 0;
        let mut result = [0; 32];
        for i in 0..self.len {
            result[i] = cur;
            let value = self.get_at_index(i);
            if value & 0b1000 == 0 {
                cur += 1;
            } else {
                cur -= 1;
            }
            
        }
        result[self.len] = 0;
        result
    }
    pub fn flip(&mut self) {
        for i in 0..((self.len + 1) / 2) {
            let j = self.len - 1 - i;
            let left = self.get_at_index(i);
            let right = self.get_at_index(j);
            self.replace_at_index(i, right ^ 0b1000);
            self.replace_at_index(j, left ^ 0b1000);
        }
        self.limit_front = self.len - self.limit_back;
        self.limit_back = self.len - self.limit_front;
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
        for i in (0..self.limit_back).rev() {
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
    // Shifts all events starting from index to the left, effectively removing the event at index
    fn remove_at_index(&mut self, index: usize) {
        let shift = index * 4;
        let mask = (1u128 << shift) - 1;
        let upper = self.data & !mask & (!(0b1111 << shift));
        let lower = self.data & mask ;
        self.data = upper >> 4 | lower;
        self.len -= 1;
        if index < self.limit_front {
            self.limit_front -= 1;
        }
        if index < self.limit_back {
            self.limit_back -= 1;
        }
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
    // Assumes that the segment is within the limits
    fn insert_at_indexes(&mut self, index_a: usize, value_a: u8, index_b: usize, value_b: u8) {
        self.insert_at_index(index_b, value_b);
        self.insert_at_index(index_a, value_a);
        self.limit_back += 2;
    }
}