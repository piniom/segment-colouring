#[cfg(test)]
mod test;

pub mod generate_all;
pub mod hash;
pub mod string;

pub const MAX_CLIQUE: u32 = 4;
pub const EXPECTED_COLOURS: u32 = MAX_CLIQUE * 2 - 1;

// Each `Event` is 4 bits,
// 0 - 7 for start events (with colours) (first bit is 0 for start events)
// 8 - 15 for end events (with colours) (first bit is 1 for end events)
// We can store 32 events in u128
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub data: u128,
    pub len: u8,
    pub limit_front: u8,
    pub limit_back: u8,
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
    pub fn insert_segment(&mut self, segment_start: u8, segment_end: u8, color: u8) {
        self.insert_at_indexes(segment_start, color, segment_end, color | 0b1000);
    }
    #[inline(always)]
    pub fn remove_segment(&mut self, segment_start: usize, segment_end: usize) {
        self.remove_at_index(segment_end);
        self.remove_at_index(segment_start);
    }
    #[inline(always)]
    pub fn move_limit_front(&mut self) {
        let first_end = self.find_first_end().unwrap();
        dbg!(first_end, self.len);
        self.limit_front = first_end as u8;
        dbg!(&self);
        self.remove_at_index(first_end as usize);
        dbg!(&self);
        self.remove_at_index(0);
    }
    #[inline(always)]
    pub fn move_limit_back(&mut self) {
        let last_start = self.find_last_start().unwrap();
        dbg!(last_start, self.len);
        self.limit_back = last_start as u8;
        self.remove_at_index(last_start as usize);
        self.remove_at_index(self.len as usize - 1);
    }
    #[inline(always)]
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
    #[inline(always)]
    pub fn intersection_counts(&self) -> [u32; 32] {
        let mut cur = 0;
        let mut result = [0; 32];
        for i in 0..self.len {
            result[i as usize] = cur;
            let value = self.get_at_index(i);
            if value & 0b1000 == 0 {
                cur += 1;
            } else {
                cur -= 1;
            }
        }
        result[self.len as usize] = 0;
        result
    }
    #[inline(always)]
    pub fn intersection_masks(&self) -> [u8; 32] {
        let mut cur = 0;
        let mut result = [0; 32];
        for i in 0..self.len {
            result[i as usize] = cur;
            let value = self.get_at_index(i);
            if value & 0b1000 == 0 {
                cur |= 1 << (value & 0b111);
            } else {
                cur &= !(1 << (value & 0b111));
            }
        }
        result[self.len as usize] = 0;
        result
    }
    #[inline(always)]
    pub fn colours_used(&self) -> u32 {
        self.intersection_masks()
            .iter()
            .fold(0u8, |acc, cur| acc | *cur)
            .count_ones()
    }
    #[inline(always)]
    pub fn allowed_colours(&self) -> [u8; 32] {
        let mut result = self.intersection_masks();
        result
            .iter_mut()
            // Each colour that is not an intersectee is allowed, so we flip the bits
            .for_each(|m| *m = !*m);
        result
    }
    #[inline(always)]
    // Assumes that the segment is 'proper' (i.e. there is no segment that would be entirely contained within it)
    pub fn allowed_colours_for_segment(&self, segment_start: u8, segment_end: u8) -> u8 {
        let masks = self.allowed_colours();
        masks[segment_start as usize] & masks[segment_end as usize]
    }
    #[inline(always)]
    pub fn valid_segment_ends(&self, segment_start: u8) -> (u8, u8) {
        if segment_start < self.limit_front || segment_start > self.limit_back {
            return (segment_start, segment_start);
        }
        let intersections = self.intersection_counts();
        if intersections[segment_start as usize] >= MAX_CLIQUE {
            return (segment_start, segment_start);
        }
        let mut currently_opened = 0i8;
        for i in 0..segment_start {
            currently_opened += -1 + 2 * event_is_start(self.get_at_index(i)) as i8;
        }

        let mut i = segment_start;
        while i < self.limit_back {
            if currently_opened == 0 {
                break;
            }
            if intersections[i as usize + 1] >= MAX_CLIQUE {
                return (segment_start, segment_start);
            }
            if event_is_end(self.get_at_index(i)) {
                currently_opened -= 1;
            }
            i += 1;
        }
        if currently_opened != 0 {
            return (segment_start, segment_start);
        }
        let min_end = i;
        while i < self.limit_back {
            if intersections[i as usize + 1] >= MAX_CLIQUE {
                break;
            }
            if event_is_end(self.get_at_index(i)) {
                break;
            }
            i += 1;
        }
        (min_end, i + 1)
    }
    #[inline(always)]
    pub fn flip(&mut self) {
        for i in 0..((self.len + 1) / 2) {
            let j = self.len - 1 - i;
            let left = self.get_at_index(i);
            let right = self.get_at_index(j);
            self.replace_at_index(i, right ^ 0b1000);
            self.replace_at_index(j, left ^ 0b1000);
        }
        self.limit_back = self.len - self.limit_front;
    }
    #[inline(always)]
    fn find_first_end(&self) -> Option<u8> {
        for i in self.limit_front..self.len {
            if self.get_at_index(i) & 0b1000 != 0 {
                return Some(i);
            }
        }
        None
    }
    #[inline(always)]
    fn find_last_start(&self) -> Option<u8> {
        for i in (0..self.limit_back).rev() {
            if self.get_at_index(i) & 0b1000 == 0 {
                return Some(i);
            }
        }
        None
    }
    #[inline(always)]
    fn replace_at_index(&mut self, index: u8, value: u8) {
        let shift = index * 4;
        self.data &= !(0b1111 << shift);
        self.data |= (value as u128) << shift;
    }
    #[inline(always)]
    // Shifts all events starting from (index + 1) to the left, effectively removing the event at index
    fn remove_at_index(&mut self, index: usize) {
        let shift = index * 4;
        let mask = (1u128 << shift) - 1;
        let upper = self.data & !mask & (!(0b1111 << shift));
        let lower = self.data & mask;
        self.data = upper >> 4 | lower;
        self.len -= 1;
        if index < self.limit_front as usize {
            self.limit_front -= 1;
        }
        if index < self.limit_back as usize {
            self.limit_back -= 1;
        }
    }
    #[inline(always)]
    fn get_at_index(&self, index: u8) -> u8 {
        let shift = index * 4;
        ((self.data >> shift) & 0b1111) as u8
    }
    #[inline(always)]
    // Shifts all events starting from index to the right and inserts the new value at index
    fn insert_at_index(&mut self, index: u8, value: u8) {
        let shift = index * 4;
        let mask = (1u128 << shift) - 1;
        let upper = self.data & !mask;
        let lower = self.data & mask;
        self.data = upper << 4 | (value as u128) << shift | lower;
        self.len += 1;
    }
    #[inline(always)]
    // Assumes that the segment is within the limits
    fn insert_at_indexes(&mut self, index_a: u8, value_a: u8, index_b: u8, value_b: u8) {
        self.insert_at_index(index_b, value_b);
        self.insert_at_index(index_a, value_a);
        self.limit_back += 2;
    }
}

#[inline(always)]
fn event_is_start(event: u8) -> bool {
    event & 0b1000 == 0
}

#[inline(always)]
fn event_is_end(event: u8) -> bool {
    event & 0b1000 == 0b1000
}
