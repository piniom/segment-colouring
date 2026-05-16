#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    Default { depth: usize },
}

impl Depth {
    pub fn new(depth: usize) -> Self {
        Self::Default { depth }
    }
    pub fn should_continue(&self) -> bool {
        match self {
            Self::Default { depth: 0 } => false,
            Self::Default { .. } => true,
        }
    }
    pub fn decrement(&self) -> Self {
        match self {
            Self::Default { depth: 0 } => Self::Default { depth: 0 },
            Self::Default { depth } => Self::Default { depth: *depth - 1 },
        }
    }
    pub fn raw_value(&self) -> usize {
        match self {
            Self::Default { depth } => *depth,
        }
    }
}
