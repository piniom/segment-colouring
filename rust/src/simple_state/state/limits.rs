use super::*;

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    #[inline(always)]
    pub fn limit_front(&self) -> u8 {
        ((self.data >> 117) & 0b1_1111) as u8
    }

    #[inline(always)]
    pub fn limit_back(&self) -> u8 {
        ((self.data >> 122) & 0b1_1111) as u8
    }
    
    #[inline(always)]
    pub fn move_limit_front(&mut self) {
        let first_end = self.find_first_end().unwrap();
        self.set_limit_front(first_end as u8);
        self.remove_at_index(first_end as usize);
        self.remove_at_index(0);
    }
    #[inline(always)]
    pub fn move_limit_front_n_times(&mut self, n: usize) {
        for _ in 0..n {
            self.move_limit_front();
        }
    }
    #[inline(always)]
    pub fn move_limit_back(&mut self) {
        let last_start = self.find_last_start().unwrap();
        self.set_limit_back(last_start as u8);
        self.remove_at_index(last_start as usize);
        self.remove_at_index(self.len() as usize - 1);
    }
    #[inline(always)]
    pub fn try_moved_limit_front(&self) -> Option<Self> {
        let first_end = self.find_first_end()?;
        let mut clone = *self;
        clone.set_limit_front(first_end as u8);
        clone.remove_at_index(first_end as usize);
        clone.remove_at_index(0);
        Some(clone)
    }
    #[inline(always)]
    pub fn try_moved_limit_back(&self) -> Option<Self> {
        let last_start = self.find_last_start()?;
        let mut clone = *self;
        clone.set_limit_back(last_start as u8);
        clone.remove_at_index(last_start as usize);
        clone.remove_at_index(self.len() as usize - 1);
        Some(clone)
    }
    #[inline(always)]
    pub fn move_limit_back_n_times(&mut self, n: usize) {
        for _ in 0..n {
            self.move_limit_back();
        }
    }
    #[inline(always)]
    pub fn move_limit_front_by_one(&mut self) {
        if self.get_at_index(self.limit_front()) & 0b1000 != 0 {
            self.move_limit_front();
        } else {
            self.set_limit_front(self.limit_front() + 1);
        }
    }
    #[inline(always)]
    pub fn front_moved(&self) -> Self {
        let mut cloned = *self;
        cloned.move_limit_front_by_one();
        cloned
    }

    #[inline(always)]
    pub fn back_moved(&self) -> Self {
        let mut cloned = *self;
        cloned.move_limit_back_by_one();
        cloned
    }
    #[inline(always)]
    pub fn move_limit_back_by_one(&mut self) {
        if self.get_at_index(self.limit_back() - 1) & 0b1000 == 0 {
            self.move_limit_back();
        } else {
            self.set_limit_back(self.limit_back() - 1);
        }
    }
    // limit_front: bits 117-121
    #[inline(always)]
    pub(super) fn set_limit_front(&mut self, value: u8) {
        let value = value & 0b1_1111;
        self.data &= !(0b1_1111 << 117);
        self.data |= (value as u128) << 117;
    }
    // limit_back: bits 122-126
    #[inline(always)]
    pub(super) fn set_limit_back(&mut self, value: u8) {
        let value = value & 0b1_1111;
        self.data &= !(0b1_1111 << 122);
        self.data |= (value as u128) << 122;
    }

    #[inline(always)]
    fn find_first_end(&self) -> Option<u8> {
        self.find_nth_end(1)
    }

    #[inline(always)]
    fn find_last_start(&self) -> Option<u8> {
        self.find_nth_start(1)
    }

    #[inline(always)]
    fn find_nth_start(&self, n: usize) -> Option<u8> {
        let mut count = 0;
        for i in (0..self.limit_back()).rev() {
            if self.get_at_index(i) & 0b1000 == 0 {
                count += 1;
                if count == n {
                    return Some(i);
                }
            }
        }
        None
    }
    #[inline(always)]
    fn find_nth_end(&self, n: usize) -> Option<u8> {
        let mut count = 0;
        for i in self.limit_front()..self.len() {
            if self.get_at_index(i) & 0b1000 != 0 {
                count += 1;
                if count == n {
                    return Some(i);
                }
            }
        }
        None
    }
}
