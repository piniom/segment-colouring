use super::*;

#[test]
fn test_state() {
    let mut state = State::new();
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
    let mut state = State::new();
    state.insert_at_indexes(0, 1, 0, 2);
    state.insert_at_indexes(0, 3, 1, 4);
    assert_eq!(state.get_at_index(0), 3);
    assert_eq!(state.get_at_index(1), 1);
    assert_eq!(state.get_at_index(2), 4);
    assert_eq!(state.get_at_index(3), 2);
}

#[test]
fn test_state_remove_at_index() {
    let mut state = State::new();
    state.insert_at_indexes(0, 1, 0, 2);
    state.insert_at_indexes(0, 3, 1, 4);
    state.remove_at_index(1);

    assert_eq!(state.get_at_index(0), 3);
    assert_eq!(state.get_at_index(1), 4);
    assert_eq!(state.get_at_index(2), 2);
    assert_eq!(state.len, 3);
}

#[test]
fn test_state_remove_at_index_2() {
    let mut state = State::from_string("[ABCDabcd]");
    state.remove_at_index(4);
    assert_eq!(state.to_string(), "[ABCDbcd]");
}

#[test]
fn test_state_remove_at_index_3() {
    let mut state = State::from_string("[ABCDabcd]");
    state.remove_at_index(7);
    assert_eq!(state, State::from_string("[ABCDabc]"));
}

#[test]
fn test_state_flip() {
    let mut state = State::new();
    state.insert_at_indexes(0, 1, 0, 2);
    state.insert_at_indexes(0, 3, 1, 4);
    let mut clone = state.clone();
    clone.flip();
    clone.flip();
    assert_eq!(state, clone);
}

#[test]
fn test_state_flip_2() {
    let mut state = State::from_string("[ABab]");
    state.flip();
    assert_eq!(state.to_string(), "[BAba]");
}

#[test]
fn test_limit_front() {
    let mut state = State::from_string("[ABCDabcd]");
    assert_eq!(state.to_string(), "[ABCDabcd]");
    state.move_limit_front();
    assert_eq!(state.to_string(), "BCD[bcd]");
}

#[test]
fn test_limit_back() {
    let mut state = State::from_string("[ABCDabcd]");
    state.move_limit_back();
    assert_eq!(state, State::from_string("[ABC]abc"));
}

#[test]
fn test_intersections() {
    let state = State::from_string("A[BCDabcdA]a");
    assert_eq!(state.intersections().to_vec(), [0, 1, 2, 3, 4, 3, 2, 1, 0, 1, 0, 1]);
}