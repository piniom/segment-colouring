use segment_colouring::simple_state::{
    find::{search_state::SearchState, FindResult},
    state::State,
};

const MAX_CLIQUE: u32 = 5;
const DEPTH: u8 = 255;
const MAX_SIZE: u8 = 11;
const FILENAME: &str = "out5.txt";

fn main() {
    println!("Out: {FILENAME} (MQ: {MAX_CLIQUE}, D: {DEPTH}, MS: {MAX_SIZE})");
    let state = State::<MAX_CLIQUE>::new();
    let search_state = SearchState::new();
    let result = state.find_strategy_root(&search_state, DEPTH, MAX_SIZE);
    println!("\n\n{:?}", result);
    println!("Visited states: {}", search_state.map.len());
    if let FindResult::Winning { .. } = result {
        
        let mut file = std::fs::File::create(FILENAME).unwrap();
        state.print_strategy(&search_state, &mut file);
        // let mut file = std::fs::File::create("draw.tex").unwrap();
        // let graph = state.graph_strategy(&search_state);
        // graph.print_tikz(&mut file).unwrap();
    }
}
