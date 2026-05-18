use crate::simple_state::state::State;
use chrono::Utc;
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

    fn format_time_now() -> String {
        Utc::now().format("%H:%M %d/%m").to_string()
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
            "{:<6} {:<12} {:<14} {:<10} {:<22} {:<22} {:<22} {}",
            "depth",
            "time",
            "elapsed",
            "result",
            "w_states",
            "delta",
            "w_substates",
            "t_states/t_substates"
        );
        for d in 3..=depth {
            print!("{:<6} {:<12} ", d, Self::format_time_now());
            let _ = io::stdout().flush();
            let start = Instant::now();
            let result = self.find_strategy(search_state, d, max_size);
            let elapsed = Self::format_elapsed(start.elapsed());
            let result_label = match result {
                FindResult::Winning => "win",
                FindResult::Losing => "lose",
            };
            print!("{:<14} {:<10} ", elapsed, result_label);
            let _ = io::stdout().flush();
            let stats = search_state.count_stats();
            let delta = stats.winning_states - previous_winning;

            println!(
                "{:<22} {:<22} {:<22} {}/{}",
                stats.winning_states,
                delta,
                stats.winning_substates,
                stats.total_states,
                stats.total_substates
            );
            previous_winning = stats.winning_states;
            if let w @ FindResult::Winning = result {
                println!(
                    "{:<6} {:<12} {:<14}",
                    "total",
                    Self::format_time_now(),
                    Self::format_elapsed(total_start.elapsed())
                );
                return w;
            }
        }
        println!(
            "{:<6} {:<12} {:<14}",
            "total",
            Self::format_time_now(),
            Self::format_elapsed(total_start.elapsed())
        );
        FindResult::Losing
    }
}
