use clap::Parser;
mod simple_state;

use simple_state::{
    find::{search_state::SearchState, FindResult},
    state::State,
};

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
}

fn run<const MAX_CLIQUE: u32>(args: &Args) {
    println!(
        "Out: {} (MQ: {MAX_CLIQUE}, D: {}, MS: {})",
        args.output, args.depth, args.max_size
    );
    let state = State::<MAX_CLIQUE>::new();
    let search_state = SearchState::new();
    let result = state.find_strategy_root(&search_state, args.depth, args.max_size);
    println!("\n\n{:?}", result);
    println!("Visited states: {}", search_state.map.len());
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
