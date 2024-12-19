use std::collections::{HashMap, HashSet};

use crate::{
    axis::{Axis, SegmentId},
    utils::find_lowest_missing,
};

pub type ColourId = u8;

#[derive(Debug, Clone, Default)]
pub struct FirstFitColourer {
    axis: Axis,
    colours: HashMap<SegmentId, ColourId>,
}

impl FirstFitColourer {
    pub fn insert_segment(&mut self, start_index: usize, end_index: usize) -> Option<SegmentId> {
        let id = self.axis.insert_segment(start_index, end_index)?;
        let colliding_colours =
            self.colliding_colours(self.axis.segment_collides_with(id).unwrap());
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
        self.axis.to_string(&self.colours)
    }
    pub fn axis(&self) -> &Axis {
        &self.axis
    }
    pub fn colours(&self) -> &HashMap<SegmentId, ColourId> {
        &self.colours
    }
}
