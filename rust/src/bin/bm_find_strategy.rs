use segment_colouring::simple_state::{
    find::{FindStateResult, SearchState, Visited},
    state::State,
};

fn main() {
    let state = State::<3>::new();
    let mut search_state = SearchState::default();
    let result = state.find_strategy(&mut search_state, 7, 7);
    for (ws, (bf, bb)) in search_state.map.iter().filter_map(|(ws, v)| {
        if let Visited::Winning{barrier, ..} = v {
            Some((ws.limits_as_barriers(barrier), ws.barrier_to_limits(barrier)))
        } else {
            None
        }
    }) {
        println!("{ws} [{bf}, {bb}]")
    }
    println!("\n\n{:?}", result);
    println!("Visited states: {}", search_state.map.len());
    println!(
        "Winning states {}",
        search_state
            .map
            .values()
            .filter(|v| if let Visited::Winning{..} = v {
                true
            } else {
                false
            })
            .count()
    );
    if let FindStateResult::True{..} = result {
        let mut file = std::fs::File::create("out.txt").unwrap();
        state.print_strategy(&search_state, &mut file);
        let mut file = std::fs::File::create("draw.tex").unwrap();
        let graph = state.graph_strategy(&search_state);
        graph.print_tikz(&mut file).unwrap();
    }
}
