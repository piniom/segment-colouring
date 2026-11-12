use super::compressed::CompressedState;
use super::*;

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
        assert!(
            !has_bits_beyond_len(&child),
            "stray bits after back reduction of {state}"
        );
        assert!(
            child == expected,
            "try_moved_limit_back != move_limit_back for {state}"
        );
    }
    if let Some(child) = state.try_moved_limit_front() {
        let mut expected = *state;
        expected.move_limit_front();
        assert!(
            !has_bits_beyond_len(&child),
            "stray bits after front reduction of {state}"
        );
        assert!(
            child == expected,
            "try_moved_limit_front != move_limit_front for {state}"
        );
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
        assert_eq!(
            CompressedState::<7>::new(once),
            CompressedState::<7>::new(twice)
        );
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

/// Every state reachable from the empty state by inserts (any allowed colour) and
/// reductions, up to `max_size` segments. Includes all limit positions reductions produce.
pub(super) fn reachable_states(max_size: u8) -> Vec<State<4>> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    let mut stack = vec![State::<4>::new()];
    while let Some(state) = stack.pop() {
        if !seen.insert(state.data) {
            continue;
        }
        result.push(state);
        stack.extend([state.try_moved_limit_front(), state.try_moved_limit_back()].into_iter().flatten());
        if state.size() < max_size {
            for m in state.moves() {
                stack.extend(m.outcomes());
            }
        }
    }
    result
}

/// Random well-formed states (proper segments, at most 12) with random colours and limits.
pub(super) fn random_states(count: usize) -> Vec<State<4>> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(7);
    let mut result = Vec::new();
    while result.len() < count {
        let mut state = State::<4>::new();
        for _ in 0..rng.random_range(0..=12) {
            let moves: Vec<_> = state.moves().map(|m| m.move_).collect();
            let Some(&m) = moves.get(rng.random_range(0..moves.len().max(1))) else {
                break;
            };
            state.insert_segment(m.0, m.1, rng.random_range(0..7));
        }
        let len = state.len();
        let front = rng.random_range(0..=len);
        state.set_limit_front(front);
        state.set_limit_back(rng.random_range(front..=len));
        result.push(state);
    }
    result
}

/// The original (slow, event-by-event) normalisation, kept as the reference.
fn reference_normalize_inner<const Q: u32>(state: &mut State<Q>, with_flip: bool) -> bool {
    let mut color_map = [0u8; 15];
    let mut next_color = 1;
    for i in 0..state.len() {
        let value = state.get_at_index(i);
        if value & 0b1000 == 0 {
            if color_map[value as usize] == 0 {
                color_map[value as usize] = next_color;
                next_color += 1;
            }
            state.replace_at_index(i, color_map[value as usize] - 1);
        } else {
            state.replace_at_index(i, color_map[(value & 0b111) as usize].wrapping_sub(1) | 0b1000);
        }
    }
    if !with_flip {
        return false;
    }
    let mut flipped = *state;
    reference_flip(&mut flipped);
    reference_normalize_inner(&mut flipped, false);
    let ev_mask: u128 = 0x0000FFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    if flipped.data & ev_mask < state.data & ev_mask {
        *state = flipped;
        true
    } else {
        false
    }
}

fn reference_flip<const Q: u32>(state: &mut State<Q>) {
    let len = state.len();
    let mut new_data: u128 = 0;
    for i in 0..len {
        let event = ((state.data >> ((len - 1 - i) * 4)) & 0b1111) as u8;
        new_data |= ((event ^ 0b1000) as u128) << (i * 4);
    }
    state.set_data(new_data);
    let old_limit_back = state.limit_back();
    state.set_limit_back(state.len() - state.limit_front());
    state.set_limit_front(state.len() - old_limit_back);
}

fn reference_is_symmetric<const Q: u32>(state: &State<Q>) -> bool {
    let mut flipped = *state;
    reference_flip(&mut flipped);
    reference_normalize_inner(&mut flipped, false);
    flipped.data() == state.data()
}

#[test]
fn fast_normalisation_matches_reference() {
    let mut states = reachable_states(6);
    let reachable = states.len();
    states.extend(random_states(20_000));
    for state in &states {
        for with_flip in [true, false] {
            let (mut fast, mut slow) = (*state, *state);
            let fast_flipped = fast.normalize_inner(with_flip);
            let slow_flipped = reference_normalize_inner(&mut slow, with_flip);
            assert_eq!(fast_flipped, slow_flipped, "{state} with_flip={with_flip}");
            assert!(fast == slow, "{state}: {fast} != {slow}");
        }
        let (mut fast, mut slow) = (*state, *state);
        fast.flip();
        reference_flip(&mut slow);
        assert!(fast == slow, "flip {state}: {fast} != {slow}");
        let mut norm = *state;
        norm.normalize();
        assert_eq!(norm.is_symmetric(), reference_is_symmetric(&norm), "{norm}");
    }
    assert!(reachable > 10_000, "only {reachable} reachable states");
}
