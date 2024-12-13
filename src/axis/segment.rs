#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct Segment {
    start_index: Option<usize>,
    end_index: Option<usize>,
}

impl Segment {
    pub fn new(start_index: usize, end_index: usize) -> Self {
        Segment {
            start_index: Some(start_index),
            end_index: Some(end_index),
        }
    }
    pub fn left(end_index: usize) -> Self {
        Segment {
            start_index: None,
            end_index: Some(end_index),
        }
    }
    pub fn right(start_index: usize) -> Self {
        Segment {
            start_index: Some(start_index),
            end_index: None,
        }
    }
    pub fn shift_start(&mut self, shift: isize) {
        let mut start = match self.start_index() {
            Some(s) => s,
            None => return,
        };
        if shift >= 0 {
            start += shift as usize;
        } else {
            start -= (-shift) as usize;
        }
        self.set_start(start)
    }
    pub fn remove_start(&mut self) {
        self.start_index = None;
    }
    pub fn remove_end(&mut self) {
        self.end_index = None;
    }
    pub fn shift_end(&mut self, shift: isize) {
        let mut end = match self.end_index() {
            Some(s) => s,
            None => return,
        };
        if shift >= 0 {
            end += shift as usize;
        } else {
            end -= (-shift) as usize;
        }
        self.set_end(end)
    }
    pub fn set_start(&mut self, start_index: usize) {
        self.start_index = Some(start_index)
    }
    pub fn set_end(&mut self, end_index: usize) {
        self.end_index = Some(end_index)
    }
    pub fn starts_before(&self, other: usize) -> bool {
        match self.start_index {
            Some(s) => s < other,
            None => true,
        }
    }
    pub fn start_index(&self) -> Option<usize> {
        self.start_index
    }
    pub fn end_index(&self) -> Option<usize> {
        self.end_index
    }
    pub fn has_both_events(&self) -> bool {
        self.start_index.is_some() && self.end_index.is_some()
    }
}