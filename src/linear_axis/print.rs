use std::collections::BTreeSet;

use super::{Event, LinearAxis};

impl LinearAxis {
    pub fn to_string(&self) -> String {
        let mut colours = BTreeSet::new();
        for e in &self.events {
            colours.insert(e.colour());
        }
        colours
            .into_iter()
            .rev()
            .map(|c| self.colour_to_string(c))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
            + &self.axis_to_string()
    }

    fn colour_to_string(&self, colour: u8) -> String {
        let mut s = format!("{:2}: ", colour);
        let mut active = self
            .events
            .iter()
            .filter(|e| e.colour() == colour)
            .next()
            .map(Event::is_start)
            == Some(false);
        for e in &self.events {
            if e.colour() == colour {
                if e.is_start() {
                    active = true;
                    s += &format!(" ⊢")
                } else {
                    active = false;
                        s += "⊣ "
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
        for (i, _) in self.events.iter().enumerate() {
            s += &format!("{:2}", i);
        }
        s
    }
}
