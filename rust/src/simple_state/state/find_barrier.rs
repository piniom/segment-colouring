use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FindBarrier {
    pub front: u8,
    pub back: u8,
}

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn limits_to_barrier(&self) -> FindBarrier {
        FindBarrier::new(self.limit_front(), self.len() - self.limit_back())
    }
    pub fn representative_barrier(&self, barrier: FindBarrier) -> FindBarrier {
        FindBarrier::new(
            self.limit_front() + barrier.front,
            self.len() - self.limit_back() + barrier.back,
        )
    }
}

impl FindBarrier {
    fn new(front: u8, back: u8) -> Self {
        Self { front, back }
    }
    pub fn flip(&self) -> Self {
        Self::new(self.back, self.front)
    }
}

impl PartialOrd for FindBarrier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(std::cmp::Ordering::Equal)
        } else if self.front <= other.front && self.back <= other.back {
            Some(std::cmp::Ordering::Greater)
        } else if self.front >= other.front && self.back >= other.back {
            Some(std::cmp::Ordering::Less)
        } else {
            None
        }
    }
}
