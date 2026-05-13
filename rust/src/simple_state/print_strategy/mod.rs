use std::io::Write;

use ahash::{HashSet, HashSetExt};

use crate::simple_state::{
    find::search_state::{FindStatus, Reduction, SearchState, WinningMove},
    state::{representative::Representative, State},
    Move,
};

// pub mod graph;

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
        let FindStatus::Winning(move_) = search_state.get_knowledge(&norm).status else {
            let repr = Representative::new(norm);
            dbg!(search_state.map.get(&repr));
            dbg!(search_state.map.get(&Representative::new(*self)));
            panic!(
                "Strategy incomplete! {} {:?}",
                &norm,
                search_state.get_knowledge(&norm).status
            );
        };
        let move_str = match move_ {
            WinningMove::Move(Move(l, r)) => format!("{l} {r}"),
            WinningMove::Reduction(Reduction::Front) => ">".to_string(),
            WinningMove::Reduction(Reduction::Back) => "<".to_string(),
        };
        if printed.contains(&norm) {
            return;
        }
        writeln!(w, "{} {}", norm.to_string(), move_str).unwrap();
        printed.insert(norm);
        match move_ {
            WinningMove::Move(move_) => {
                for child in norm.with_move(move_).outcomes() {
                    // println!("move");
                    child.print_strategy_inner(search_state, w, printed);
                }
            }
            WinningMove::Reduction(Reduction::Front) => {
                norm.move_limit_front();
                // println!("limit_front");
                norm.print_strategy_inner(search_state, w, printed);
            }
            WinningMove::Reduction(Reduction::Back) => {
                norm.move_limit_back();
                // println!("limit_front");
                norm.print_strategy_inner(search_state, w, printed);
            }
        }
    }
}
