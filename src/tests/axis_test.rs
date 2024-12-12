use crate::axis::{Axis, Event, Segment};

#[test]
fn axis_adding_segments_test() {
    let mut axis = Axis::default();

    axis.insert_segment(0, 0).unwrap();
    assert_eq!(axis.events(), [Event::Start(0), Event::End(0)]);
    assert_eq!(axis.segments(), vec![Segment::new(0, 1)]);

    axis.insert_segment(0, 0).unwrap();
    assert_eq!(
        [
            Event::Start(1),
            Event::End(1),
            Event::Start(0),
            Event::End(0)
        ],
        axis.events()
    );
    assert_eq!(axis.possible_ends(2), 2..=3);
    assert_eq!(axis.possible_ends(0), 0..=1);
    assert_eq!(
        axis.segments(),
        vec![Segment::new(2, 3), Segment::new(0, 1)]
    );

    axis.insert_segment(1, 3).unwrap();
    assert_eq!(
        [
            Event::Start(1),
            Event::Start(2),
            Event::End(1),
            Event::Start(0),
            Event::End(2),
            Event::End(0)
        ],
        axis.events()
    );
    assert_eq!(axis.possible_ends(2), 5..=5);
    assert_eq!(
        axis.segments(),
        vec![Segment::new(3, 5), Segment::new(0, 2), Segment::new(1, 4),]
    );
}

#[test]
fn axis_removing_segments_test() {
    let mut axis = Axis::default();

    let old_events = axis.events().to_vec();
    let old_segments = axis.segments();
    let id = axis.insert_segment(0, 0).unwrap();
    let mut copy = axis.clone();
    copy.remove_segment(id);
    assert_eq!(old_events, copy.events());
    assert_eq!(old_segments, copy.segments());

    let old_events = axis.events().to_vec();
    let old_segments = axis.segments();
    let id = axis.insert_segment(0, 0).unwrap();
    let mut copy = axis.clone();
    copy.remove_segment(id);
    assert_eq!(old_events, copy.events());
    assert_eq!(old_segments, copy.segments());

    let old_events = axis.events().to_vec();
    let old_segments = axis.segments();
    let id = axis.insert_segment(1, 3).unwrap();
    let mut copy = axis.clone();
    copy.remove_segment(id);
    assert_eq!(old_events, copy.events());
    assert_eq!(old_segments, copy.segments());
}
