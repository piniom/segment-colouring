use std::collections::{BTreeSet, HashMap};

use crate::{
    axis::{Axis, Event, SegmentId},
    first_fit::ColourId,
};

impl Axis {
    pub fn to_string(&self, colours: &HashMap<SegmentId, ColourId>) -> String {
        let mut colours = colours.clone();
        for s in self.segments.keys() {
            colours.entry(*s).or_insert(100);
        }
        colours
            .values()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .rev()
            .map(|c| self.colour_to_string(*c, &colours))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
            + &self.axis_to_string()
    }

    fn colour_to_string(&self, colour: ColourId, colours: &HashMap<SegmentId, ColourId>) -> String {
        let mut s = format!("{:2}: ", colour);
        let mut active = false;
        for e in self.events() {
            if *colours.get(&e.segment_id()).unwrap() == colour {
                match e {
                    Event::Start(_) => {
                        active = true;
                        s += &format!("{:2}", e.segment_id())
                    }
                    Event::End(_) => {
                        active = false;
                        s += "⊣ "
                    }
                }
            } else if active {
                s += "--"
            } else {
                s += "  "
            }
        }
        s
    }
    fn axis_to_string(&self) -> String {
        let mut s = "id: ".to_string();
        for (i, _) in self.events().iter().enumerate() {
            s += &format!("{:2}", i);
        }
        s
    }
}
