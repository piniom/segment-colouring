use std::io::Write;

use ahash::HashMap;

use crate::simple_state::{find::Visited, state::State};

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE>  {
    pub fn print_strategy(&self, map: &HashMap<Self, Visited>, w: &mut impl Write) {
        writeln!(w, "{} {}", MAX_CLIQUE, Self::EXPECTED_COLOURS).unwrap();
        self.print_strategy_inner(map, w);
    }
    fn print_strategy_inner(&self, map: &HashMap<Self, Visited>, w: &mut impl Write) {
        let mut norm = *self;
        norm.normalize();
        let Some(Visited::Winning(move_)) = map.get(&norm) else {
            panic!("Strategy incomplete! {}", &norm)
        };
        writeln!(w, "{} {} {}", norm.to_string(), move_.0, move_.1).unwrap();
        for child in norm.with_move(*move_).outcomes() {
            child.print_strategy_inner(map, w);
        }
    }
}