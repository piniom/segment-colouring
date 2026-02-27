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
    assert_eq!(state.allowed_colours_for_segment(0, 4), 0b11110000);
    assert_eq!(state.allowed_colours_for_segment(4, 10), 0b11110000);
    assert_eq!(state.allowed_colours_for_segment(8, 8), !0);
    assert_eq!(state.allowed_colours_for_segment(9, 9), !0 - 1);
    assert_eq!(state.allowed_colours_for_segment(10, 10), !0);
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
    let max_clique = state.max_clique() as u8;
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
