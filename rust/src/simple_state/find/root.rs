use crate::simple_state::state::State;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use super::{FindResult, SearchState};

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    fn format_elapsed(elapsed: Duration) -> String {
        let secs = elapsed.as_secs_f64();
        if secs >= 3600.0 {
            format!("{:.2}h", secs / 3600.0)
        } else if secs >= 60.0 {
            format!("{:.2}m", secs / 60.0)
        } else if secs >= 1.0 {
            format!("{:.2}s", secs)
        } else {
            format!("{}ms", elapsed.as_millis())
        }
    }

    pub fn find_strategy_root(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let mut previous_winning = 0;
        let total_start = Instant::now();
        println!();
        println!(
            "{:<5} {:<12} {:<8} {:<20} {:<20} {}",
            "depth", "elapsed", "result", "winning", "delta", "w_substates"
        );
        for d in 3..=depth {
            print!("{:<5} ", d);
            let _ = io::stdout().flush();
            let start = Instant::now();
            let result = self.find_strategy(search_state, d, max_size);
            let elapsed = Self::format_elapsed(start.elapsed());
            let result_label = match result {
                FindResult::Winning => "win",
                FindResult::Losing => "lose",
            };
            print!("{:<12} {:<8} ", elapsed, result_label);
            let _ = io::stdout().flush();
            let (cur_winning, substates) = search_state.count_winning();
            let delta = cur_winning - previous_winning;

            println!(
                "{:<20} {:<20} {}",
                cur_winning,
                delta,
                substates
            );
            previous_winning = cur_winning;
            if let w @ FindResult::Winning = result {
                println!(
                    "{:<5} {:<12} {:<8} {:<20} {:<20} {}",
                    "total",
                    Self::format_elapsed(total_start.elapsed()),
                    "",
                    "",
                    "",
                    ""
                );
                return w;
            }
        }
        println!(
            "{:<5} {:<12} {:<8} {:<20} {:<20} {}",
            "total",
            Self::format_elapsed(total_start.elapsed()),
            "",
            "",
            "",
            ""
        );
        FindResult::Losing
    }
}
