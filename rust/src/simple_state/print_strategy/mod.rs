use std::io::Write;

use ahash::{HashSet, HashSetExt};

use crate::simple_state::{
    find::{SearchState, Visited},
    state::State,
};

pub mod graph;

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn print_strategy(&self, search_state: &SearchState<MAX_CLIQUE>, w: &mut impl Write) {
        writeln!(w, "{} {}", MAX_CLIQUE, Self::EXPECTED_COLOURS).unwrap();
        self.print_strategy_inner(search_state, w, &mut HashSet::new());
    }
    fn print_strategy_inner(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        w: &mut impl Write,
        printed: &mut HashSet<State<MAX_CLIQUE>>,
    ) {
        let mut norm = *self;
        norm.normalize();
        let Some(Visited::Winning { move_, .. }) = search_state.map.get(&norm) else {
            panic!("Strategy incomplete! {}", &norm)
        };
        if printed.contains(&norm) {
            return;
        }
        writeln!(w, "{} {} {}", norm.to_string(), move_.0, move_.1).unwrap();
        printed.insert(norm);
        for child in norm.with_move(*move_).outcomes() {
            child.print_strategy_inner(search_state, w, printed);
        }
    }
}
