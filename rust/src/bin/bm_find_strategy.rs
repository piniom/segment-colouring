use segment_colouring::simple_state::{
    find::{search_state::SearchState, FindResult},
    state::State,
};

fn main() {
    let state = State::<3>::new();
    let mut search_state = SearchState::default();
    let result = state.find_strategy_root(&mut search_state, 10, 8);
    println!("\n\n{:?}", result);
    println!("Visited states: {}", search_state.map.len());
    if let FindResult::Winning { .. } = result {
        let mut file = std::fs::File::create("out.txt").unwrap();
        state.print_strategy(&search_state, &mut file);
        // let mut file = std::fs::File::create("draw.tex").unwrap();
        // let graph = state.graph_strategy(&search_state);
        // graph.print_tikz(&mut file).unwrap();
    }
}
