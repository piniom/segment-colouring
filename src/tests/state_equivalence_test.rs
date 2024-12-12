use std::collections::HashMap;

use crate::{
    axis::SegmentId,
    state_equivalence::{CompressedEvent, EventType, NormalizedState},
};

#[test]
fn test_normalize_single_event() {
    let state = vec![(EventType::Start, 1)];
    let colouring = HashMap::from([(1, 100)]);

    let result = NormalizedState::normalize(state, colouring);
    let expected = NormalizedState(vec![CompressedEvent::Start(0)]);

    assert_eq!(result, expected);
}

#[test]
fn test_normalize_multiple_events_same_colour() {
    let state = vec![
        (EventType::Start, 1),
        (EventType::End, 1),
        (EventType::Start, 1),
    ];
    let colouring = HashMap::from([(1, 100)]);

    let result = NormalizedState::normalize(state, colouring);
    let expected = NormalizedState(vec![
        CompressedEvent::Start(0),
        CompressedEvent::End,
        CompressedEvent::Start(0),
    ]);

    assert_eq!(result, expected);
}

#[test]
fn test_normalize_multiple_events_different_colours() {
    let state = vec![
        (EventType::Start, 1),
        (EventType::Start, 2),
        (EventType::End, 1),
        (EventType::End, 2),
    ];
    let colouring = HashMap::from([(1, 100), (2, 200)]);

    let result = NormalizedState::normalize(state, colouring);
    let expected = NormalizedState(vec![
        CompressedEvent::Start(0),
        CompressedEvent::Start(1),
        CompressedEvent::End,
        CompressedEvent::End,
    ]);

    assert_eq!(result, expected);
}

#[test]
fn test_normalize_no_events() {
    let state: Vec<(EventType, SegmentId)> = vec![];
    let colouring = HashMap::new();

    let result = NormalizedState::normalize(state, colouring);
    let expected = NormalizedState(vec![]);

    assert_eq!(result, expected);
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn test_normalize_missing_colour() {
    let state = vec![(EventType::Start, 1)];
    let colouring = HashMap::new();

    // This should panic because the colour map does not contain the required SegmentId.
    NormalizedState::normalize(state, colouring);
}
