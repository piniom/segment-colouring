use crate::{
    state::State,
    stats::{self, Snapshot},
};
use chrono::Local;
use serde_json::json;
use std::thread;
use std::time::{Duration, Instant};
use std::{io::Write, sync::mpsc};

use super::{FindResult, SearchState};

/// Where per-depth JSON records are written (one JSON object per line).
pub type StatsSink = Box<dyn Write + Send>;

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    fn spawn_printer<'scope, 'env>(
        scope: &'scope thread::Scope<'scope, 'env>,
        rx: mpsc::Receiver<CountMessage>,
        search_state: &'env SearchState<MAX_CLIQUE>,
        mut stats_sink: Option<StatsSink>,
    ) -> thread::ScopedJoinHandle<'scope, ()>
    where
        'env: 'scope,
    {
        scope.spawn(move || {
            let mut previous_winning = 0;
            print_headers();
            for message in rx {
                match message {
                    CountMessage::Row {
                        depth,
                        time,
                        elapsed,
                        duration,
                        result_label,
                        counters,
                    } => {
                        print!(
                            "{:<6} {:<12} {:<14} {:<10} ",
                            depth, time, elapsed, result_label
                        );
                        let _ = std::io::stdout().flush();
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
                        if stats::ENABLED {
                            println!("       {}", counters.summary_line());
                        }
                        if let Some(sink) = stats_sink.as_mut() {
                            // Map stats are sampled after the iteration, while the next one
                            // may already be adding entries; counters are exact per depth.
                            let mut record = json!({
                                "type": "depth",
                                "depth": depth,
                                "result": result_label,
                                "elapsed_ms": duration.as_secs_f64() * 1000.0,
                                "map": {
                                    "states": stats.total_states,
                                    "substates": stats.total_substates,
                                    "winning_states": stats.winning_states,
                                    "winning_substates": stats.winning_substates,
                                },
                            });
                            if stats::ENABLED {
                                record["map"]["entries_per_key"] = json!(stats.entries_per_key);
                                record["map"]["states_by_size"] = json!(stats.states_by_size);
                                record["counters"] = counters.to_json();
                            }
                            let written = writeln!(sink, "{record}").and_then(|_| sink.flush());
                            if let Err(e) = written {
                                eprintln!("warning: could not write stats record: {e}");
                            }
                        }
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

    fn format_elapsed_precise(elapsed: std::time::Duration) -> String {
        let secs = elapsed.as_secs();
        let millis = elapsed.subsec_millis();
        if secs >= 3600 {
            format!(
                "{}:{:02}:{:02}.{:03}",
                secs / 3600,
                (secs % 3600) / 60,
                secs % 60,
                millis
            )
        } else if secs >= 60 {
            format!("{:02}:{:02}.{:03}", secs / 60, secs % 60, millis)
        } else {
            format!("{}.{:03}s", secs, millis)
        }
    }

    fn format_time_now() -> String {
        Local::now().format("%H:%M %d.%m").to_string()
    }

    pub fn find_strategy_root(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
        stats_sink: Option<StatsSink>,
    ) -> FindResult {
        let total_start = Instant::now();
        let (tx, rx) = mpsc::channel::<CountMessage>();
        let mut final_result = FindResult::Losing;

        thread::scope(|s| {
            let printer = Self::spawn_printer(s, rx, search_state, stats_sink);
            let mut previous_counters = stats::snapshot();

            for d in 3..=depth {
                let time = Self::format_time_now();
                let start = Instant::now();
                let result = self.find_strategy(search_state, d, max_size);
                let duration = start.elapsed();
                let now_counters = stats::snapshot();
                let counters = now_counters.since(&previous_counters);
                previous_counters = now_counters;
                let elapsed = chrono::Duration::from_std(duration)
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
                    duration,
                    result_label,
                    counters,
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
            Self::format_elapsed_precise(total_start.elapsed())
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
        depth: u8,
        time: String,
        elapsed: String,
        duration: Duration,
        result_label: String,
        /// Counters accumulated during this depth's iteration.
        counters: Snapshot,
    },
}
