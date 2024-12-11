use crate::axis::{Axis, Event};

#[test]
fn axis_adding_segments_test() {
    let mut axis = Axis::default();
    axis.insert_segment(0, 0).unwrap();
    assert_eq!(axis.events(), [Event::Start(0), Event::End(0)]);
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
}

#[test]
fn axis_removing_segments_test() {
    let mut axis = Axis::default();

    let old_events = axis.events().to_vec();
    let id = axis.insert_segment(0, 0).unwrap();
    let mut copy = axis.clone();
    copy.remove_segment(id);
    assert_eq!(old_events, copy.events());

    let old_events = axis.events().to_vec();
    let id = axis.insert_segment(0, 0).unwrap();
    let mut copy = axis.clone();
    copy.remove_segment(id);
    assert_eq!(old_events, copy.events());

    let old_events = axis.events().to_vec();
    let id = axis.insert_segment(1, 3).unwrap();
    let mut copy = axis.clone();
    copy.remove_segment(id);
    assert_eq!(old_events, copy.events());
}

