use futures::future::join_all;

use super::{normalization::{NormalizedState, StrategyNormalizer}, History, LinearAxis};

#[derive(Debug, Clone)]
pub struct ClicquedLinearAxis {
    pub inner: LinearAxis,
    pub max_clicque: usize,
    pub intersections: Vec<usize>,
    pub normalizer: StrategyNormalizer
}

impl ClicquedLinearAxis {
    pub fn new(max_clicque: usize) -> Self {
        Self::with_inner(LinearAxis::new(), max_clicque)
    }

    pub fn with_inner(inner: LinearAxis, max_clicque: usize) -> Self {
        let mut result = Self {
            inner,
            max_clicque,
            intersections: vec![],
            normalizer: StrategyNormalizer::new()
        };
        result.count_intersections();
        result
    }

    pub fn apply_history(&mut self, history: History) -> Option<History> {
        let reverse = self.inner.apply_history(history, self.max_colors());
        self.count_intersections();
        reverse
    }
    pub async fn generate_all_states_async(
        &mut self,
        depth: isize,
        async_depth: isize,
    ) -> Vec<NormalizedState> {
        if depth == 0 {
            return vec![self.strategy_normalize_without_symmetry()].into();
        }

        if async_depth <= 0 {
            let mut cloned = self.clone();
            return tokio::spawn(async move {cloned.generate_all_states(depth)}).await.unwrap();
        }

        

        let ends = self.valid_new_segment_ends(0).unwrap();

        let mut futures = vec![];
        let colors_used = self.colours_used();
        for e in ends.0..=ends.1 {
            let colors = self.uncollisions(0, e).into_iter().filter(|&t| t as usize <= colors_used);
            for c in colors {
                let mut cloned_self = self.clone();

                let handle = async move {
                    cloned_self
                        .apply_history(History::SegmentInsert {
                            start_index: 0,
                            end_index: e,
                            color: c,
                        })
                        .unwrap();

                    cloned_self.generate_all_states_async(depth - 1, async_depth - 1).await
                };
                futures.push(handle);
            }
        }

        join_all(futures).await.into_iter().flatten().collect()
    }

    pub fn generate_all_states(&mut self, depth: isize) -> Vec<NormalizedState> {
        if depth == 0 {
            return vec![self.strategy_normalize_without_symmetry()];
        }
        let ends = self.valid_new_segment_ends(0).unwrap();
        let mut states = vec![];
        let colors_used = self.colours_used();
        for e in ends.0..=ends.1 {
            let colors = self.uncollisions(0, e).into_iter().filter(|&t| t as usize <= colors_used);
            for c in colors {
                let rev = self
                    .apply_history(History::SegmentInsert {
                        start_index: 0,
                        end_index: e,
                        color: c,
                    })
                    .unwrap();
                states.extend(self.generate_all_states(depth - 1));
                self.apply_history(rev);
            }
        }
        states
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

    pub fn segments_opened_at_front(&self) -> usize {
        let mut opened = vec![false; self.max_colors()];
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

    pub fn valid_new_segments<'a>(&'a self) -> Vec<(usize, usize)> {
        self.valid_new_segment_starts()
            .filter_map(|s| {
                self.valid_new_segment_ends(s)
                    .map(|(min_end, max_end)| (min_end..=max_end).map(move |e| (s, e)))
            })
            .flatten()
            .collect()
    }

    pub fn valid_new_segment_ends(&self, start: usize) -> Option<(usize, usize)> {
        let mut opened_before = self.segments_opened_at_front();
        let evs = &self.inner.events;
        let mut iter = evs.into_iter();
        for e in iter.by_ref().take(start){
            if e.is_start() {
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
            if !iter.next().unwrap().is_start() {
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
            if !iter.next().unwrap().is_start() {
                break;
            }
            i += 1
        }
        Some((min_end, i))
    }

    pub fn segment_will_collide_with_colours(&self, start: usize, end: usize) -> Vec<bool> {
        let mut collisions = vec![false; self.max_colors()];
        for e in self.inner.events.iter().skip(start).take(start - end) {
            collisions[e.colour() as usize] = true
        }
        collisions
    }

    fn valid_new_segment_starts<'a>(&'a self) -> impl Iterator<Item = usize> + use<'a> {
        (0..self.intersections.len())
            .filter(|&i| i == self.intersections.len() || self.intersections[i] < self.max_clicque)
    }
    pub fn colours_used(&self) -> usize {
        let mut used = vec![false; self.max_colors()];
        for e in &self.inner.events {
            used[e.colour() as usize] = true
        }
        used.into_iter().filter(|u| *u).count()
    }
    pub fn max_colors(&self) -> usize {
        self.max_clicque * 2 - 1
    }
    pub fn uncollisions(&self, start: usize, end: usize) -> Vec<u8> {
        self.segment_will_collide_with_colours(start, end)
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if *c { None } else { Some(i as u8) })
            .collect()
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
        .into_iter()
        .map(|(s, e)| (s, e, axis.segment_will_collide_with_colours(s, e)))
        .collect::<Vec<_>>());
    println!("{}", axis.inner.to_string());
    dbg!(axis.intersections);
}
