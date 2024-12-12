use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    axis::{Axis, Event, SegmentId},
    utils::find_lowest_missing,
};

pub type ColourId = u32;

#[derive(Debug, Clone, Default)]
pub struct FirstFitColourer {
    axis: Axis,
    colours: HashMap<SegmentId, ColourId>,
}

impl FirstFitColourer {
    pub fn insert_segment(&mut self, start_index: usize, end_index: usize) -> Option<SegmentId> {
        let id = self.axis.insert_segment(start_index, end_index)?;
        let colliding_colours =
            self.colliding_colours(self.axis.segment_collides_with(id).unwrap().into_iter());
        let colour = find_lowest_missing(colliding_colours);
        self.colours.insert(id, colour);
        Some(id)
    }
    pub fn colliding_colours(
        &self,
        segments: impl IntoIterator<Item = SegmentId>,
    ) -> impl Iterator<Item = ColourId> {
        let set: HashSet<_> = segments
            .into_iter()
            .map(|s| *self.colours.get(&s).unwrap())
            .collect();
        set.into_iter()
    }
    pub fn to_string(&self) -> String {
        self.colours
            .values()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .rev()
            .map(|c| self.colour_to_string(*c))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
            + &self.axis_to_string()
    }
    pub fn axis(&self) -> &Axis {
        &self.axis
    }
    pub fn colours(&self) -> &HashMap<SegmentId, ColourId> {
        &self.colours
    }
    fn colour_to_string(&self, colour: ColourId) -> String {
        let mut s = format!("{:2}: ", colour);
        let mut active = false;
        for e in self.axis.events() {
            if *self.colours.get(&e.segment_id()).unwrap() == colour {
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
            } else {
                if active {
                    s += "--"
                } else {
                    s += "  "
                }
            }
        }
        s
    }
    fn axis_to_string(&self) -> String {
        let mut s = format!("id: ");
        for (i, _) in self.axis.events().iter().enumerate() {
            s += &format!("{:2}", i);
        }
        s
    }
}
