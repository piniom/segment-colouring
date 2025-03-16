use std::collections::BTreeSet;

use super::{clicqued::ClicquedLinearAxis, Event, LinearAxis};

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

impl ClicquedLinearAxis {
    pub fn strategy_string(&self) -> String {
        let mut started = vec![false; self.max_colours];
        let mut front = vec![];
        for e in &self.inner.events {
            if e.is_start() {
                started[e.colour() as usize] = true;
            } else if !started[e.colour() as usize] {
                started[e.colour() as usize] = true;
                front.push(e.sibling());
            }
        }
        let mut finished = vec![false; self.max_colours];
        let mut back = vec![];
        for e in self.inner.events.iter().rev() {
            if !e.is_start() {
                finished[e.colour() as usize] = true;
            } else if !finished[e.colour() as usize] {
                finished[e.colour() as usize] = true;
                back.push(e.sibling());
            }
        }
        let middle: String = self.inner.events.iter().map(Event::print_char).collect();
        front.iter().map(Event::print_char).collect::<String>()
            + "["
            + &middle
            + "]"
            + &back.iter().rev().map(Event::print_char).collect::<String>()
    }
}
