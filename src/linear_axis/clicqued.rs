use super::{History, LinearAxis};

#[derive(Debug, Clone)]
pub struct ClicquedLinearAxis {
    pub inner: LinearAxis,
    pub max_clicque: u32,
    pub max_colours: usize,
    pub intersections: Vec<u32>,
}

impl ClicquedLinearAxis {
    pub fn new(max_clicque: u32) -> Self {
        Self {
            inner: LinearAxis::new(),
            max_clicque,
            max_colours: (max_clicque * 2 - 1) as usize,
            intersections: vec![],
        }
    }

    pub fn apply_history(&mut self, history: History) -> Option<History> {
        let reverse = self.inner.apply_history(history);
        self.count_intersections();
        reverse
    }

    fn count_intersections(&mut self) {
        let mut current = self.segments_opened_at_front();
        let mut result = vec![];

        for e in &self.inner.events {
            result.push(current);
            if e.is_start() {
                current += 1
            } else {
                current -= 1
            }
        }
        result.push(current);
        self.intersections = result;
    }

    pub fn segments_opened_at_front(&self) -> u32 {
        let mut opened = vec![false; self.max_colours];
        let mut result = 0;
        for e in &self.inner.events {
            if e.is_start() {
                opened[e.colour() as usize] = true;
            } else if !opened[e.colour() as usize] {
                result += 1;
            }
        }
        result
    }

    pub fn valid_new_segments<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + use<'a> {
        self.valid_new_segment_starts()
            .filter_map(|s| {
                self.valid_new_segment_ends(s)
                    .map(|(min_end, max_end)| (min_end..=max_end).map(move |e| (s, e)))
            })
            .flatten()
    }

    pub fn valid_new_segment_ends(&self, start: usize) -> Option<(usize, usize)> {
        let mut opened_before = self.segments_opened_at_front();
        let evs = &self.inner.events;
        for i in 0..start {
            if evs[i].is_start() {
                opened_before += 1
            } else {
                opened_before -= 1
            }
        }
        let mut i = start;
        while i < evs.len() {
            if opened_before == 0 {
                break;
            }
            if self.intersections[i + 1] >= self.max_clicque {
                return None;
            }
            if !evs[i].is_start() {
                opened_before -= 1
            }
            i += 1
        }
        if opened_before != 0 {
            return None;
        }
        let min_end = i;
        while i < evs.len() {
            if self.intersections[i + 1] >= self.max_clicque {
                break;
            }
            if !evs[i].is_start() {
                break
            } 
            i += 1
        }
        Some((min_end, i))
    }

    pub fn segment_will_collide_with_colours(&self, start: usize, end: usize) -> Vec<bool> {
        let mut collisions = vec![false; self.max_colours];
        for i in start..end {
            collisions[self.inner.events[i].colour() as usize] = true
        }
        collisions
    }

    fn valid_new_segment_starts<'a>(&'a self) -> impl Iterator<Item = usize> + use<'a> {
        (0..self.intersections.len())
            .filter(|&i| i == self.intersections.len() || self.intersections[i] < self.max_clicque)
    }
    pub fn colours_used(&self) -> usize {
        let mut used = vec![false; self.max_colours];
        for e in &self.inner.events {
            used[e.colour() as usize] = true
        }
        used.into_iter().filter(|u| *u).count()
    }
}

#[test]
fn test_clicqued_linear_axis() {
    let mut axis = ClicquedLinearAxis::new(3);
    axis.apply_history(History::SegmentInsert {
        start_index: 0,
        end_index: 0,
        color: 3,
    });
    axis.apply_history(History::SegmentInsert {
        start_index: 1,
        end_index: 2,
        color: 2,
    });
    axis.apply_history(History::LimitFront);
    axis.apply_history(History::LimitBack);
    dbg!(axis
        .valid_new_segments()
        .map(|(s, e)| (s, e, axis.segment_will_collide_with_colours(s, e)))
        .collect::<Vec<_>>());
    println!("{}", axis.inner.to_string());
    dbg!(axis.intersections);
}
