use crate::simple_state::state::State;
use chrono::Local;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use super::{FindResult, SearchState};

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    fn spawn_printer<'scope, 'env>(
        scope: &'scope thread::Scope<'scope, 'env>,
        rx: mpsc::Receiver<CountMessage>,
        search_state: &'env SearchState<MAX_CLIQUE>,
    ) -> thread::ScopedJoinHandle<'scope, ()>
    where
        'env: 'scope,
    {
        scope.spawn(|| {
            let mut previous_winning = 0;
            print_headers();
            for message in rx {
                match message {
                    CountMessage::Row {
                        depth,
                        time,
                        elapsed,
                        result_label,
                    } => {
                        let stats = search_state.count_stats();
                        let delta = stats.winning_states - previous_winning;
                        print!(
                            "{:<6} {:<12} {:<14} {:<10} ",
                            depth, time, elapsed, result_label
                        );
                        println!(
                            "{:<22} {:<22} {:<22} {}/{}",
                            stats.winning_states,
                            delta,
                            stats.winning_substates,
                            stats.total_states,
                            stats.total_substates
                        );
                        previous_winning = stats.winning_states;
                    }
                }
            }
        })
    }

    fn format_elapsed(elapsed: chrono::Duration) -> String {
        let secs = elapsed.num_seconds();
        if secs >= 3600 {
            let hours = secs / 3600;
            let minutes = (secs % 3600) / 60;
            format!("{:02}:{:02}", hours, minutes)
        } else if secs >= 60 {
            let minutes = secs / 60;
            let seconds = secs % 60;
            format!("{:02}:{:02}", minutes, seconds)
        } else if secs >= 1 {
            format!("00:{:02}", secs)
        } else {
            format!("{}ms", elapsed.subsec_millis())
        }
    }

    fn format_time_now() -> String {
        Local::now().format("%H:%M %d.%m").to_string()
    }

    pub fn find_strategy_root(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: usize,
        max_size: u8,
    ) -> FindResult {
        let total_start = Instant::now();
        let (tx, rx) = mpsc::channel::<CountMessage>();
        let mut final_result = FindResult::Losing;

        thread::scope(|s| {
            let printer = Self::spawn_printer(s, rx, search_state);

            for d in 3..=depth {
                let time = Self::format_time_now();
                let start = Instant::now();
                let result = self.find_strategy(search_state, d, max_size);
                let elapsed = chrono::Duration::from_std(start.elapsed())
                    .unwrap_or_else(|_| chrono::Duration::zero());
                let elapsed = Self::format_elapsed(elapsed);
                let result_label = match result {
                    FindResult::Winning => "win".to_string(),
                    FindResult::Losing => "lose".to_string(),
                };
                let _ = tx.send(CountMessage::Row {
                    depth: d,
                    time,
                    elapsed,
                    result_label,
                });
                if let FindResult::Winning = result {
                    final_result = FindResult::Winning;
                    break;
                }
            }
            drop(tx);
            let _ = printer.join();
        });

        println!(
            "{:<6} {:<12} {:<14}",
            "total",
            Self::format_time_now(),
            Self::format_elapsed(
                chrono::Duration::from_std(total_start.elapsed())
                    .unwrap_or_else(|_| chrono::Duration::zero())
            )
        );
        final_result
    }
}

fn print_headers() {
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
}

enum CountMessage {
    Row {
        depth: usize,
        time: String,
        elapsed: String,
        result_label: String,
    },
}
