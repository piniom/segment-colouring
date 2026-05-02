use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct FindBarrier {
    pub front: u8,
    pub back: u8,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn find_barrier(&self, front: u8, back: u8) -> FindBarrier {
        FindBarrier::new(front, self.len() - back)
    }
    pub fn barrier_to_limits(&self, barrier: &FindBarrier) -> (u8, u8) {
        (barrier.front, self.len() - barrier.back)
    }
    pub fn limits_as_barriers(&self, barrier: &FindBarrier) -> Self {
        let mut clone = *self;
        let (front, back) = self.barrier_to_limits(barrier);
        clone.set_limit_front(front);
        clone.set_limit_back(back);
        clone
    }
}

impl FindBarrier {
    fn new(front: u8, back: u8) -> Self {
        Self { front, back }
    }
    pub fn confine(&self, other: &Self) -> Self {
        Self::new(self.front.min(other.front), self.back.min(other.back))
    }
    pub fn flip(&self) -> Self {
        Self::new(self.back, self.front)
    }
}
