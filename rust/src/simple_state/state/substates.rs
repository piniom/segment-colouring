use super::*;

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn substates(&self) -> Vec<Self> {
        self.substate_combinations()
            .into_iter()
            .map(|(f, b)| {
                let mut clone = *self;
                clone.move_limit_front_n_times(f);
                clone.move_limit_back_n_times(b);
                clone
            })
            .collect()
    }
    fn substate_combinations(&self) -> Vec<(usize, usize)> {
        let before = self.segments_before_start_count();
        let after = self.segments_after_end_count();
        (0..=before)
            .flat_map(|x| (0..=after).map(move |y| (x, y)))
            .collect()
    }
    fn segments_before_start_count(&self) -> usize {
        (0..self.limit_front())
            .map(|i| self.get_at_index(i))
            .filter(|e| event_is_end(*e))
            .count()
    }
    fn segments_after_end_count(&self) -> usize {
        (self.limit_back()..self.len())
            .rev()
            .map(|i| self.get_at_index(i))
            .filter(|e| event_is_start(*e))
            .count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_segments_before_start_count() {
        let state = State::<4>::from_string("A[BCDabcdA]a");
        assert_eq!(state.segments_before_start_count(), 0);

        let state = State::<4>::from_string("[ABCDabcd]");
        assert_eq!(state.segments_before_start_count(), 0);

        let state = State::<4>::from_string("ABCD[abcd]");
        assert_eq!(state.segments_before_start_count(), 0);

        let state = State::<4>::from_string("ABCDa[bcd]");
        assert_eq!(state.segments_before_start_count(), 1);

        let state = State::<4>::from_string("ABCDabcd[Aa]");
        assert_eq!(state.segments_before_start_count(), 4);
    }

    #[test]
    fn test_segments_before_end_count() {
        let state = State::<4>::from_string("A[BCDabcdA]a");
        assert_eq!(state.segments_after_end_count(), 0);

        let state = State::<4>::from_string("[ABCDabcd]");
        assert_eq!(state.segments_after_end_count(), 0);

        let state = State::<4>::from_string("[ABCD]abcd");
        assert_eq!(state.segments_after_end_count(), 0);

        let state = State::<4>::from_string("[ABCDa]bcd");
        assert_eq!(state.segments_after_end_count(), 0);

        let state = State::<4>::from_string("[ABCDab]cd");
        assert_eq!(state.segments_after_end_count(), 0);

        let state = State::<4>::from_string("[ABCDabc]d");
        assert_eq!(state.segments_after_end_count(), 0);

        let state = State::<4>::from_string("[ABCDabcd]Aa");
        assert_eq!(state.segments_after_end_count(), 1);

        let state = State::<4>::from_string("Aa[ABCDabcd]AaAaAaBb");
        assert_eq!(state.segments_after_end_count(), 4);

        let state = State::<4>::from_string("Aa[ABCDabcd]ABCDabcd");
        assert_eq!(state.segments_after_end_count(), 4);
    }
}
