use crate::{
    find::search_state::Key,
    memory::{self, format_bytes, MemoryUsage},
    state::State,
    stats::{self, Snapshot},
};
use chrono::Local;
use serde_json::json;
use std::thread;
use std::time::{Duration, Instant};
use std::{io::Write, sync::mpsc};

use super::{
    search_state::{CountStats, StateCounts},
    FindResult, SearchState,
};

/// How the iterative deepening ended.
pub struct RootOutcome {
    pub result: FindResult,
    /// The depth at which `stop_after_quiet` ended the search, if it did.
    pub stopped_quiet: Option<u8>,
}

/// Where per-depth JSON records are written (one JSON object per line).
pub type StatsSink = Box<dyn Write + Send>;

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    fn spawn_printer<'scope, 'env>(
        scope: &'scope thread::Scope<'scope, 'env>,
        rx: mpsc::Receiver<CountMessage>,
        search_state: &'env SearchState<MAX_CLIQUE>,
        max_size: u8,
        mut stats_sink: Option<StatsSink>,
        counts_reply: Option<mpsc::Sender<CountStats>>,
    ) -> thread::ScopedJoinHandle<'scope, ()>
    where
        'env: 'scope,
    {
        scope.spawn(move || {
            let mut previous_winning = 0;
            print_headers();
            let store_at_limit = search_state.store_at_limit;
            for message in rx {
                match message {
                    CountMessage::Row {
                        depth,
                        time,
                        elapsed,
                        duration,
                        result_label,
                        counters,
                        memory,
                        map_capacity,
                    } => {
                        print!(
                            "{:<6} {:<15} {:<14} {:<10} ",
                            depth, time, elapsed, result_label
                        );
                        let _ = std::io::stdout().flush();
                        let stats = search_state.count_stats(max_size);
                        let (below, at_limit) = (stats.below_limit, stats.at_limit);
                        let delta = below.winning_states - previous_winning;
                        let map_bytes = map_table_bytes(map_capacity);
                        // Losing states at the limit are stored only with `--store-at-limit`.
                        let losing_at_limit = if store_at_limit {
                            at_limit.non_winning_states().to_string()
                        } else {
                            "-".to_string()
                        };
                        println!(
                            "{:<13} {:<13} {:<13} {:<13} {:<13} {:<25} {:<25} {:<11} {}",
                            below.winning_states,
                            delta,
                            below.states,
                            at_limit.winning_states,
                            losing_at_limit,
                            format!("{}/{}", below.winning_substates, below.substates),
                            format!("{}/{}", at_limit.winning_substates, at_limit.substates),
                            memory.best_rss().map_or("-".into(), format_bytes),
                            format_bytes(map_bytes),
                        );
                        previous_winning = below.winning_states;
                        if let Some(reply) = &counts_reply {
                            let _ = reply.send(stats);
                        }
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
                                "memory": {
                                    "rss_bytes": memory.rss_bytes,
                                    "peak_rss_bytes": memory.peak_rss_bytes,
                                    "map_capacity": map_capacity,
                                    "map_table_bytes": map_bytes,
                                },
                                "map": {
                                    // Totals over both groups (the fields before the split).
                                    "states": below.states + at_limit.states,
                                    "substates": below.substates + at_limit.substates,
                                    "winning_states": below.winning_states + at_limit.winning_states,
                                    "winning_substates": below.winning_substates
                                        + at_limit.winning_substates,
                                    "below_limit": counts_json(&below),
                                    "below_limit_winning_delta": delta,
                                    "at_limit": counts_json(&at_limit),
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
        Local::now().format("%H:%M:%S %d.%m").to_string()
    }

    /// Iterative deepening over depths `3..=depth`. With `stop_after_quiet: Some(n)` it
    /// also stops (losing) after `n` consecutive depths that changed neither the number of
    /// winning states nor the number of stored states.
    pub fn find_strategy_root(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        depth: u8,
        max_size: u8,
        stats_sink: Option<StatsSink>,
        stop_after_quiet: Option<u8>,
    ) -> RootOutcome {
        let total_start = Instant::now();
        let (tx, rx) = mpsc::channel::<CountMessage>();
        let mut final_result = FindResult::Losing;
        let mut stopped_quiet = None;

        thread::scope(|s| {
            // With `stop_after_quiet`, the printer sends its per-depth map counts back, and
            // the loop waits for them before the next depth (so they are exact). Without
            // it, the printer counts while the next depth already runs.
            let (counts_tx, counts_rx) = match stop_after_quiet {
                Some(_) => {
                    let (t, r) = mpsc::channel::<CountStats>();
                    (Some(t), Some(r))
                }
                None => (None, None),
            };
            let printer = Self::spawn_printer(s, rx, search_state, max_size, stats_sink, counts_tx);
            let mut previous_counters = stats::snapshot();
            let mut previous_counts: Option<CountStats> = None;
            let mut quiet_depths = 0u8;

            for d in 3..=depth {
                let start = Instant::now();
                let result = self.find_strategy(search_state, d, max_size);
                let duration = start.elapsed();
                // When this depth finished (not when it started).
                let time = Self::format_time_now();
                // Cheap (no pass over the entries), so read right when the depth ends.
                let memory = memory::current();
                let map_capacity = search_state.map.capacity();
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
                    memory,
                    map_capacity,
                });
                if let FindResult::Winning = result {
                    final_result = FindResult::Winning;
                    break;
                }
                if let (Some(limit), Some(counts_rx)) = (stop_after_quiet, &counts_rx) {
                    let Ok(counts) = counts_rx.recv() else { break };
                    let quiet = previous_counts.is_some_and(|p| is_quiet(&p, &counts));
                    quiet_depths = if quiet { quiet_depths + 1 } else { 0 };
                    previous_counts = Some(counts);
                    if quiet_depths >= limit {
                        stopped_quiet = Some(d);
                        break;
                    }
                }
            }
            drop(tx);
            let _ = printer.join();
        });

        println!(
            "{:<6} {:<15} {:<14}",
            "total",
            Self::format_time_now(),
            Self::format_elapsed_precise(total_start.elapsed())
        );
        if let Some(d) = stopped_quiet {
            println!(
                "Stopped at depth {d}: {} quiet depths in a row",
                stop_after_quiet.unwrap_or_default()
            );
        }
        RootOutcome { result: final_result, stopped_quiet }
    }
}

/// A depth is quiet when it changed no state count: neither the stored states nor the
/// winning states, below the size limit or at it. (Entry counts may still change.)
fn is_quiet(previous: &CountStats, current: &CountStats) -> bool {
    let same = |p: &StateCounts, c: &StateCounts| {
        p.states == c.states && p.winning_states == c.winning_states
    };
    same(&previous.below_limit, &current.below_limit) && same(&previous.at_limit, &current.at_limit)
}

fn counts_json(counts: &StateCounts) -> serde_json::Value {
    json!({
        "states": counts.states,
        "substates": counts.substates,
        "winning_states": counts.winning_states,
        "winning_substates": counts.winning_substates,
    })
}

/// Columns: states below the size limit (`w_below` winning, `delta` its change, `t_below`
/// all), then states at the limit (`w_limit` winning, `l_limit` losing, stored only with
/// `--store-at-limit`), then winning/all entries in each group.
fn print_headers() {
    println!();
    println!(
        "{:<6} {:<15} {:<14} {:<10} {:<13} {:<13} {:<13} {:<13} {:<13} {:<25} {:<25} {:<11} {}",
        "depth",
        "finished",
        "elapsed",
        "result",
        "w_below",
        "delta",
        "t_below",
        "w_limit",
        "l_limit",
        "w/t_sub_below",
        "w/t_sub_limit",
        if cfg!(target_os = "linux") { "rss" } else { "peak_rss" },
        "map_table"
    );
}

/// Bytes of the map's hash table: one key/value slot plus one control byte per bucket.
fn map_table_bytes(capacity: usize) -> u64 {
    let slot = std::mem::size_of::<(Key, super::search_state::PackedKnowledge)>() + 1;
    (capacity * slot) as u64
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
        /// Process memory and map size when the iteration ended.
        memory: MemoryUsage,
        map_capacity: usize,
    },
}

#[cfg(test)]
mod quiet_test {
    use super::*;

    /// MQ=4, MS=7 is a loss whose state count stops growing early: `stop_after_quiet`
    /// must end the search by itself, and the loss must verify.
    #[test]
    fn stops_after_quiet_depths() {
        let _lock = super::super::SEARCH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let search_state = SearchState::<4>::new();
        let root = State::<4>::new();
        let outcome = root.find_strategy_root(&search_state, u8::MAX, 7, None, Some(3));
        assert!(matches!(outcome.result, FindResult::Losing));
        let stopped = outcome.stopped_quiet.expect("did not stop by itself");
        assert!(stopped < 40, "stopped only at depth {stopped}");
        assert!(root.verify_loss(&search_state, 7).is_verified());
    }
}
