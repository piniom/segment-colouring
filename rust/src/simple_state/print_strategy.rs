use std::io::Write;

use crate::simple_state::{find::{Visited, SearchState}, state::State};

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE>  {
    pub fn print_strategy(&self, search_state: &SearchState<MAX_CLIQUE>, w: &mut impl Write) {
        writeln!(w, "{} {}", MAX_CLIQUE, Self::EXPECTED_COLOURS).unwrap();
        self.print_strategy_inner(search_state, w);
    }
    fn print_strategy_inner(&self, search_state: &SearchState<MAX_CLIQUE>, w: &mut impl Write) {
        let mut norm = *self;
        norm.normalize();
        let Some(Visited::Winning(move_)) = search_state.map.get(&norm) else {
            panic!("Strategy incomplete! {}", &norm)
        };
        writeln!(w, "{} {} {}", norm.to_string(), move_.0, move_.1).unwrap();
        for child in norm.with_move(*move_).outcomes() {
            child.print_strategy_inner(search_state, w);
        }
    }
}