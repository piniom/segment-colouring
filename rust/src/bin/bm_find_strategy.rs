use segment_colouring::simple_state::{find::{Visited, SearchState}, state::State};

fn main() {
    let state = State::<3>::new();
    let mut search_state = SearchState::default();
    let result = state.find_strategy(&mut search_state, 20, 7);
    println!("{:?}", result);
    println!("Visited states: {}", search_state.map.len());
    println!(
        "Winning states {}",
        search_state.map.values()
            .filter(|v| if let Visited::Winning(_) = v {
                true
            } else {
                false
            })
            .count()
    );
    if result {
        let mut file = std::fs::File::create("out.txt").unwrap();
        state.print_strategy(&search_state, &mut file);
        let mut file = std::fs::File::create("draw.tex").unwrap();
        let graph = state.graph_strategy(&search_state);
        graph.print_tikz(&mut file).unwrap();
    }
}
