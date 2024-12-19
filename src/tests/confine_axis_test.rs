use crate::axis::{Axis, Event, Segment};

#[test]
fn axis_empty_confine_test() {
    let mut axis = Axis::default();

    axis.insert_segment(0, 0).unwrap();
    axis.confine(0..0);
    assert_eq!(&[] as &[Event], axis.events());
    assert_eq!(vec![] as Vec<Segment>, axis.segments());

    axis.insert_segment(0, 0).unwrap();
    axis.insert_segment(0, 0).unwrap();
    axis.insert_segment(1, 3).unwrap();
    axis.confine(0..0);
    assert_eq!(&[] as &[Event], axis.events());
    assert_eq!(vec![] as Vec<Segment>, axis.segments());
}

#[test]
fn axis_confine_test() {
    let mut axis = Axis::default();

    let inputs = [(0, 0), (0, 0), (1, 3), (2, 5), (6, 8)];
    for (s, e) in inputs {
        axis.insert_segment(s, e).unwrap();
    }
    axis.confine(2..8);
    assert_eq!(
        &[
            Event::Start(3),
            Event::End(1),
            Event::Start(0),
            Event::End(2),
            Event::Start(4),
            Event::End(3)
        ] as &[Event],
        axis.events()
    );
    assert_eq!(
        vec![
            Segment::right(4),
            Segment::left(3),
            Segment::left(5),
            Segment::new(2, 7),
            Segment::right(6)
        ] as Vec<Segment>,
        axis.segments()
    );
}

#[test]
fn axis_confine_left_right_test() {
    let mut axis = Axis::default();

    let inputs = [(0, 0), (0, 0), (1, 3), (2, 5), (6, 8)];
    for (s, e) in inputs {
        axis.insert_segment(s, e).unwrap();
    }
    let mut clone = axis.clone();
    assert!(axis.confine_left());
    assert!(clone.confine(1..10));
    assert_eq!(axis.segments, clone.segments);
    assert_eq!(axis.events(), clone.events());
    assert!(axis.confine_right());
    assert!(clone.confine(0..8));
    assert_eq!(axis.segments, clone.segments);
    assert_eq!(axis.events(), clone.events());
}
