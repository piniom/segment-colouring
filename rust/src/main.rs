use clap::Parser;
mod find;
mod moves;
mod print_strategy;
mod state;
mod stats;

use crate::{
    find::{root::StatsSink, search_state::SearchState, FindResult},
    state::State,
};
use serde_json::json;
use std::{io::Write, time::Instant};

#[derive(Parser, Debug)]
#[command(about = "Search for a winning strategy and write it to a file")]
struct Args {
    /// Maximum clique size (1..=5)
    #[arg(short = 'q', long, default_value_t = 4, value_parser = clap::value_parser!(u32).range(1..=5))]
    max_clique: u32,

    /// Search depth
    #[arg(short, long, default_value_t = 255)]
    depth: u8,

    /// Maximum state size
    #[arg(short = 's', long, default_value_t = 8)]
    max_size: u8,

    /// Output file for the strategy
    #[arg(short, long, default_value = "strategy.txt")]
    output: String,

    /// Write per-depth statistics as JSON lines to this file. Counters and detailed map
    /// statistics are included only when built with `--features stats`.
    #[arg(long)]
    stats_json: Option<String>,

    /// Also store losing states at the size limit. By default they're re-evaluated from
    /// their reductions instead, which uses ~5-14x less memory at the same speed, but gives
    /// larger strategy files. See `SearchState::store_at_limit`.
    #[arg(long)]
    store_at_limit: bool,

    /// Reserve room for about this many stored states up front (e.g. the count of a
    /// previous run). Avoids growing the map, which leaves freed tables behind. 0 = grow
    /// as needed.
    #[arg(long, default_value_t = 0)]
    capacity: usize,

    /// Label stored in the statistics file, to tell runs apart when comparing them
    #[arg(long, default_value = "")]
    label: String,
}

fn write_record(file: &mut std::fs::File, record: serde_json::Value) {
    if let Err(e) = writeln!(file, "{record}").and_then(|_| file.flush()) {
        eprintln!("warning: could not write stats record: {e}");
    }
}

fn run<const MAX_CLIQUE: u32>(args: &Args) {
    println!(
        "Out: {} (MQ: {MAX_CLIQUE}, D: {}, MS: {})",
        args.output, args.depth, args.max_size
    );
    let mut stats_file = args.stats_json.as_ref().map(|path| {
        std::fs::File::create(path)
            .unwrap_or_else(|e| panic!("cannot create stats file {path}: {e}"))
    });
    if let Some(file) = stats_file.as_mut() {
        write_record(
            file,
            json!({
                "type": "run",
                "label": args.label,
                "mq": MAX_CLIQUE,
                "ms": args.max_size,
                "max_depth": args.depth,
                "threads": rayon::current_num_threads(),
                "store_at_limit": args.store_at_limit,
                "capacity": args.capacity,
                "stats_enabled": stats::ENABLED,
                "started": chrono::Local::now().to_rfc3339(),
            }),
        );
    }
    let sink: Option<StatsSink> = stats_file
        .as_ref()
        .map(|f| Box::new(f.try_clone().expect("cannot clone stats file handle")) as StatsSink);

    let start = Instant::now();
    let state = State::<MAX_CLIQUE>::new();
    let search_state = SearchState::with_capacity(args.capacity).with_store_at_limit(args.store_at_limit);
    let result = state.find_strategy_root(&search_state, args.depth, args.max_size, sink);
    let total = start.elapsed();
    println!("\n\n{:?}", result);
    println!("Visited states: {}", search_state.map.len());
    if let Some(file) = stats_file.as_mut() {
        write_record(
            file,
            json!({
                "type": "end",
                "result": match result { FindResult::Winning => "win", FindResult::Losing => "lose" },
                "total_ms": total.as_secs_f64() * 1000.0,
                "visited_states": search_state.map.len(),
                "map_capacity": search_state.map.capacity(),
                "strategy_file": matches!(result, FindResult::Winning).then_some(&args.output),
            }),
        );
    }
    if let FindResult::Winning { .. } = result {
        let mut file = std::fs::File::create(&args.output).unwrap();
        state.print_strategy(&search_state, &mut file);
        // let mut file = std::fs::File::create("draw.tex").unwrap();
        // let graph = state.graph_strategy(&search_state);
        // graph.print_tikz(&mut file).unwrap();
    }
}

fn main() {
    let args = Args::parse();
    match args.max_clique {
        1 => run::<1>(&args),
        2 => run::<2>(&args),
        3 => run::<3>(&args),
        4 => run::<4>(&args),
        5 => run::<5>(&args),
        _ => unreachable!("max_clique validated by clap to be in 1..=5"),
    }
}
