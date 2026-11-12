use super::*;
use super::compressed::CompressedState;

#[test]
fn test_state() {
    let mut state = State::<4>::new();
    state.insert_at_index(0, 1);
    state.insert_at_index(1, 2);
    state.insert_at_index(2, 3);
    state.insert_at_index(3, 4);
    assert_eq!(state.get_at_index(0), 1);
    assert_eq!(state.get_at_index(1), 2);
    assert_eq!(state.get_at_index(2), 3);
    assert_eq!(state.get_at_index(3), 4);
}
#[test]
fn test_state_insert_at_indexes() {
    let mut state = State::<4>::new();
    state.insert_at_indexes(0, 1, 0, 2);
    state.insert_at_indexes(0, 3, 1, 4);
    assert_eq!(state.get_at_index(0), 3);
    assert_eq!(state.get_at_index(1), 1);
    assert_eq!(state.get_at_index(2), 4);
    assert_eq!(state.get_at_index(3), 2);
}

#[test]
fn test_state_remove_at_index() {
    let mut state = State::<4>::new();
    state.insert_at_indexes(0, 1, 0, 2);
    state.insert_at_indexes(0, 3, 1, 4);
    state.remove_at_index(1);

    assert_eq!(state.get_at_index(0), 3);
    assert_eq!(state.get_at_index(1), 4);
    assert_eq!(state.get_at_index(2), 2);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_state_remove_at_index_2() {
    let mut state = State::<4>::from_string("[ABCDabcd]");
    state.remove_at_index(4);
    assert_eq!(state.to_string(), "[ABCDbcd]");
}

#[test]
fn test_state_remove_at_index_3() {
    let mut state = State::<4>::from_string("[ABCDabcd]");
    state.remove_at_index(7);
    assert_eq!(state, State::<4>::from_string("[ABCDabc]"));
}

#[test]
fn test_state_flip() {
    let mut state = State::<4>::new();
    state.insert_at_indexes(0, 1, 0, 2);
    state.insert_at_indexes(0, 3, 1, 4);
    let mut clone = state.clone();
    clone.flip();
    clone.flip();
    assert_eq!(state, clone);
}

#[test]
fn test_state_flip_2() {
    let mut state = State::<4>::from_string("[ABab]");
    state.flip();
    assert_eq!(state.to_string(), "[BAba]");
}

#[test]
fn test_state_flip_limits() {
    let mut state = State::<4>::from_string("A[Ba]b");
    state.flip();
    assert_eq!(state.to_string(), "B[Ab]a");
}

#[test]
fn test_state_flip_limits_2() {
    let mut state = State::<4>::from_string("A[BCa]bc");
    state.flip();
    assert_eq!(state.to_string(), "CB[Acb]a");
}

#[test]
fn test_limit_front() {
    let mut state = State::<4>::from_string("[ABCDabcd]");
    assert_eq!(state.to_string(), "[ABCDabcd]");
    state.move_limit_front();
    assert_eq!(state.to_string(), "BCD[bcd]");
}

#[test]
fn test_limit_back() {
    let mut state = State::<4>::from_string("[ABCDabcd]");
    state.move_limit_back();
    assert_eq!(state, State::from_string("[ABC]abc"));
}

#[test]
fn test_intersection_counts() {
    let state = State::<4>::from_string("A[BCDabcdA]a");
    let mut expected = vec![0, 1, 2, 3, 4, 3, 2, 1, 0, 1, 0];
    expected.extend(vec![0; 32 - expected.len()]);
    assert_eq!(state.intersection_counts().to_vec(), expected);
}

#[test]
fn test_intersection_counts_2() {
    let state = State::<4>::from_string("A[BCDabcBdAb]a");
    let mut expected = vec![0, 1, 2, 3, 4, 3, 2, 1, 2, 1, 2, 1, 0];
    expected.extend(vec![0; 32 - expected.len()]);
    assert_eq!(state.intersection_counts().to_vec(), expected);
}

#[test]
fn test_intersection_masks() {
    let state = State::<4>::from_string("A[BCDabcdA]a");
    let mut expected = vec![
        0b0, 0b1, 0b11, 0b111, 0b1111, 0b1110, 0b1100, 0b1000, 0b0, 0b1, 0b0,
    ];
    expected.extend(vec![0; 32 - expected.len()]);
    assert_eq!(state.intersection_masks().to_vec(), expected);
}

#[test]
fn test_intersection_masks_counts() {
    let state = State::<4>::from_string("A[BCDabcdA]a");
    assert_eq!(
        state
            .intersection_masks()
            .iter()
            .map(|v| v.count_ones())
            .collect::<Vec<_>>(),
        state.intersection_counts().to_vec()
    );
}

#[test]
fn test_allowed_colours_for_segment() {
    let state = State::<4>::from_string("A[BCDabcdA]a");
    assert_eq!(state.allowed_colours_for_segment_bits(0, 4), 0b11110000);
    assert_eq!(state.allowed_colours_for_segment_bits(4, 10), 0b11110000);
    assert_eq!(state.allowed_colours_for_segment_bits(8, 8), !0);
    assert_eq!(state.allowed_colours_for_segment_bits(9, 9), !0 - 1);
    assert_eq!(state.allowed_colours_for_segment_bits(10, 10), !0);
}

#[test]
fn test_allowed_segment_ends_empty() {
    let state = State::<4>::from_string("[]");
    assert_eq!(state.valid_segment_ends(0), (0, 1))
}

#[test]
fn test_allowed_segment_ends_one() {
    let state = State::<4>::from_string("[Aa]");
    assert_eq!(state.valid_segment_ends(0), (0, 2));
    assert_eq!(state.valid_segment_ends(1), (2, 3));
    assert_eq!(state.valid_segment_ends(2), (2, 3));
}

#[test]
fn test_allowed_segment_ends_two() {
    let state = State::<4>::from_string("[AaBb]");
    assert_eq!(state.valid_segment_ends(0), (0, 2));
    assert_eq!(state.valid_segment_ends(1), (2, 4));
    assert_eq!(state.valid_segment_ends(2), (2, 4));
    assert_eq!(state.valid_segment_ends(3), (4, 5));
    assert_eq!(state.valid_segment_ends(4), (4, 5));
}

#[test]
fn test_allowed_segment_ends_two_limits() {
    let state = State::<4>::from_string("[AaB]b");
    assert_eq!(state.valid_segment_ends(0), (0, 2));
    assert_eq!(state.valid_segment_ends(1), (2, 4));
    assert_eq!(state.valid_segment_ends(2), (2, 4));
    assert_eq!(state.valid_segment_ends(3), (3, 3));
    assert_eq!(state.valid_segment_ends(4), (4, 4));
}

#[test]
fn test_allowed_segment_ends_clique() {
    let mut state = State::<5>::from_string("[]");
    let max_clique = 5u8;
    for i in 0..max_clique {
        state.insert_segment(i, i * 2, i as u8);
    }
    dbg!(state);
    assert_eq!(state.valid_segment_ends(0), (0, max_clique));
    for i in 1..=max_clique {
        assert_eq!(state.valid_segment_ends(i), (i, i));
    }
    for i in max_clique + 1..2 * max_clique {
        assert_eq!(
            state.valid_segment_ends(i),
            (2 * max_clique, 2 * max_clique + 1)
        )
    }
}

#[test]
fn test_compressed_roundtrip() {
    let state = State::<4>::from_string("[ABab]");
    let compressed = CompressedState::<2>::from(state);
    let decompressed: State<4> = compressed.into();
    assert_eq!(state, decompressed);
}

#[test]
fn test_compressed_roundtrip_more() {
    let cases = ["[]", "[Aa]", "[ABab]", "[ABCabc]", "[ABabCDcd]"];
    for s in cases {
        let state = State::<4>::from_string(s);
        let compressed = CompressedState::<3>::from(state);
        let decompressed: State<4> = compressed.into();
        assert_eq!(state, decompressed, "failed roundtrip for {s}");
    }
}



fn has_bits_beyond_len<const Q: u32>(state: &State<Q>) -> bool {
    (state.data() >> (4 * state.len() as u32)) != 0
}

fn assert_clean_reductions<const Q: u32>(state: &State<Q>) {
    if let Some(child) = state.try_moved_limit_back() {
        let mut expected = *state;
        expected.move_limit_back();
        assert!(!has_bits_beyond_len(&child), "stray bits after back reduction of {state}");
        assert!(child == expected, "try_moved_limit_back != move_limit_back for {state}");
    }
    if let Some(child) = state.try_moved_limit_front() {
        let mut expected = *state;
        expected.move_limit_front();
        assert!(!has_bits_beyond_len(&child), "stray bits after front reduction of {state}");
        assert!(child == expected, "try_moved_limit_front != move_limit_front for {state}");
    }
    for child in [state.try_moved_limit_front(), state.try_moved_limit_back()]
        .into_iter()
        .flatten()
    {
        // Lookup key (after `find_strategy` normalises) must equal store key
        // (after `update_status` normalises again).
        let mut once = child;
        once.normalize();
        let mut twice = once;
        assert!(!twice.normalize(), "normalize not idempotent for {child}");
        assert!(once == twice, "normalize not idempotent for {child}");
        assert_eq!(CompressedState::<7>::new(once), CompressedState::<7>::new(twice));
    }
}

#[test]
fn test_back_reduction_leaves_no_stray_events() {
    // Regression: `try_moved_limit_back` used the pre-removal length, leaving the
    // removed end event in the bits past `len`. That made `normalize` pick the wrong
    // orientation, so back-reduced states were looked up under a key never stored.
    assert_clean_reductions(&State::<4>::from_string("[ABabCcCDcBdbCcDd]"));
}

#[test]
fn test_reductions_clean_on_reachable_states() {
    use std::collections::HashSet;
    const MAX_SIZE: u8 = 6;
    let mut seen = HashSet::new();
    let mut stack = vec![State::<4>::new()];
    while let Some(state) = stack.pop() {
        if !seen.insert(state.data) {
            continue;
        }
        assert_clean_reductions(&state);
        let mut next: Vec<State<4>> = [state.try_moved_limit_front(), state.try_moved_limit_back()]
            .into_iter()
            .flatten()
            .collect();
        if state.size() < MAX_SIZE {
            for m in state.moves() {
                next.extend(m.outcomes());
            }
        }
        stack.extend(next);
    }
    assert!(seen.len() > 10_000, "explored only {} states", seen.len());
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "stray bits past len")]
fn test_debug_assert_catches_stray_bits() {
    let mut state = State::<4>::from_string("[ABabCcCDcBdbCc]");
    state.data |= 0xb << (4 * state.len() as u32);
    state.normalize();
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "out of bounds")]
fn test_debug_assert_catches_remove_past_len() {
    let mut state = State::<4>::from_string("[ABab]");
    state.remove_at_index(4);
}
