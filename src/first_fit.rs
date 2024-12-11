use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    axis::{Axis, Event},
    utils::find_lowest_missing,
};

#[derive(Debug, Clone, Default)]
pub struct FirstFitColourer {
    axis: Axis,
    colours: HashMap<u32, u32>,
}

impl FirstFitColourer {
    pub fn insert_segment(&mut self, start_index: usize, end_index: usize) -> Option<u32> {
        let id = self.axis.insert_segment(start_index, end_index)?;
        let colliding_colours =
            self.colliding_colours(self.axis.segment_collides_with(id).unwrap().into_iter());
        let colour = find_lowest_missing(colliding_colours);
        self.colours.insert(id, colour);
        Some(id)
    }
    pub fn colliding_colours(
        &self,
        segments: impl IntoIterator<Item = u32>,
    ) -> impl Iterator<Item = u32> {
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
            .map(|c| self.colour_to_string(*c))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
            + &self.axis_to_string()
    }
    fn colour_to_string(&self, colour: u32) -> String {
        let mut s = format!("{:2}: ", colour);
        let mut active = false;
        for e in self.axis.events() {
            if *self.colours.get(&e.segment_id()).unwrap() == colour {
                match e {
                    Event::Start(_) => {
                        active = true;
                        s += " ⊢"
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
